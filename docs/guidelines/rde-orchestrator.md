# rde-orchestrator

High-level coordination between the debug engine and extension crates.

---

## Overview

`rde-orchestrator` sits between the UI layer (REPL/CLI) and the engine layer. It provides convenient functions that combine engine operations with extension crate logic, avoiding circular dependencies.

## Why Orchestrator?

`rde-core` cannot depend on `rde-pretty-print`, `rde-tokio`, or `rde-cargo` (would create dependency cycles). The orchestrator breaks this deadlock by:
- Depending on all crates
- Exposing high-level, composable functions
- Being called from `rde-repl` and `rde-cli`

## API

### Pretty Printing

```rust
use rde_core::{DebugBackend, DebugError, ProcessHandle};
use rde_pretty_print::{FormatBudget, PrettyValue};

pub async fn pretty_print_value<B: DebugBackend>(
    backend: &B,
    handle: &ProcessHandle,
    address: u64,
    type_name: &str,
    budget: &mut FormatBudget,
) -> Result<PrettyValue, DebugError>
```

**Example:**

```rust
let mut budget = FormatBudget::default();
let value = rde_orchestrator::pretty_print_value(
    &backend, &handle, 0x140001000, "std::vec::Vec", &mut budget
).await?;
println!("{:?}", value);
```

### Tokio Task Inspection

```rust
use rde_tokio::task::AsyncTask;
use rde_core::DebugError;

pub fn list_tokio_tasks(process_id: u32) -> Result<Vec<AsyncTask>, DebugError>
```

**Example:**

```rust
let tasks = rde_orchestrator::list_tokio_tasks(1234)?;
for task in tasks {
    println!("Task {}: {:?} - {:?}", task.task_id, task.state, task.function_name);
}
```

**Return value when no Tokio runtime:**

Returns a single `AsyncTask` with:
- `task_id: 0`
- `state: Completed`
- `function_name: Some("No Tokio runtime detected")`

### Cargo Resolution & Build

```rust
use std::path::PathBuf;
use rde_cargo::CargoError;

pub async fn cargo_resolve_and_build(
    manifest_path: &PathBuf,
    package: Option<String>,
    target: Option<String>,
    profile: String,
    features: Vec<String>,
) -> Result<PathBuf, CargoError>
```

**Example:**

```rust
let manifest = PathBuf::from("./Cargo.toml");
let artifact = rde_orchestrator::cargo_resolve_and_build(
    &manifest,
    Some("my-crate".to_string()),
    Some("my-bin".to_string()),
    "dev".to_string(),
    vec!["tokio/full".to_string()],
).await?;

println!("Built artifact: {}", artifact.display());
```

**What it does:**
1. Calls `cargo metadata` to resolve the project structure
2. Finds the target binary matching the specified package/target/profile
3. Checks if the artifact is stale (mtime < newest source)
4. Runs `cargo build` if stale
5. Returns the path to the compiled artifact

## BackendMemoryReader

Internal helper that bridges the async `DebugBackend` to the synchronous `MemoryReader` trait used by pretty printers.

```rust
struct BackendMemoryReader<'a, B: DebugBackend> {
    backend: &'a B,
    handle: &'a ProcessHandle,
}

impl<'a, B: DebugBackend> MemoryReader for BackendMemoryReader<'a, B> {
    fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>, DebugError> {
        // Uses tokio::task::block_in_place to bridge async/sync
    }
}
```

> **Note:** This uses `block_in_place` as an MVP bridge. A fully async pretty-print pipeline is recommended for production.
