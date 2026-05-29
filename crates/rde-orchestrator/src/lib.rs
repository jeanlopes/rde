//! RDE Orchestrator — High-level coordination between engine and extension crates.

use rde_core::{DebugBackend, DebugError, ProcessHandle};
use rde_pretty_print::{FormatBudget, MemoryReader, PrettyValue, PrinterRegistry};
use rde_tokio::scanner::TokioScanner;
use rde_tokio::task::AsyncTask;

/// Pretty-print a value at the given address in the target process.
pub async fn pretty_print_value<B: DebugBackend>(
    backend: &B,
    handle: &ProcessHandle,
    address: u64,
    type_name: &str,
    budget: &mut FormatBudget,
) -> Result<PrettyValue, DebugError> {
    let reader = BackendMemoryReader { backend, handle };
    let registry = rde_pretty_print::built_in_registry();
    match registry.lookup(type_name) {
        Some(printer) => printer.format(&reader, address as usize, budget),
        None => Ok(PrettyValue::Raw(format!(
            "No pretty printer for type: {type_name}"
        ))),
    }
}

/// List Tokio async tasks in the target process.
pub fn list_tokio_tasks(process_id: u32) -> Result<Vec<AsyncTask>, DebugError> {
    let scanner = TokioScanner::new();
    scanner.list_tasks(process_id)
}

/// Resolve, build if stale, and return the artifact path for a Cargo project.
pub async fn cargo_resolve_and_build(
    manifest_path: &std::path::PathBuf,
    package: Option<String>,
    target: Option<String>,
    profile: String,
    features: Vec<String>,
) -> Result<std::path::PathBuf, rde_cargo::CargoError> {
    let metadata = rde_cargo::fetch_metadata(manifest_path).await?;
    let cargo_target = rde_cargo::resolve_target(&metadata, package.as_deref(), target.as_deref(), &profile, &features)?;

    let src_dir = manifest_path
        .parent()
        .map(|p| p.join("src"))
        .unwrap_or_else(|| manifest_path.clone());
    if rde_cargo::is_stale(&cargo_target, &src_dir) {
        rde_cargo::run_build(&cargo_target, manifest_path).await?;
    }

    Ok(cargo_target.artifact_path)
}

struct BackendMemoryReader<'a, B: DebugBackend> {
    backend: &'a B,
    handle: &'a ProcessHandle,
}

impl<'a, B: DebugBackend> MemoryReader for BackendMemoryReader<'a, B> {
    fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>, DebugError> {
        // Bridge async backend to synchronous MemoryReader trait.
        // In production this should be fully async; for MVP we block.
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.backend
                    .read_memory(self.handle, address as u64, len)
                    .await
            })
        })
    }
}
