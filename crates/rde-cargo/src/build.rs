//! Cargo build orchestration and staleness check.

use crate::CargoError;
use rde_core::cargo::CargoTarget;
use std::path::Path;
use std::time::SystemTime;
use tokio::process::Command;

/// Check if the artifact is stale compared to source files.
pub fn is_stale(target: &CargoTarget, src_dir: &Path) -> bool {
    let artifact_mtime = match std::fs::metadata(&target.artifact_path)
        .and_then(|m| m.modified())
    {
        Ok(t) => t,
        Err(_) => return true, // missing artifact = stale
    };

    let newest_source = match newest_file_mtime(src_dir) {
        Some(t) => t,
        None => return false, // no sources = assume fresh
    };

    newest_source > artifact_mtime
}

/// Run `cargo build` for the given target and wait for completion.
pub async fn run_build(target: &CargoTarget, manifest_path: &Path) -> Result<(), CargoError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(manifest_path);

    cmd.arg("--package").arg(&target.package_name);

    if target.target_kind == rde_core::cargo::CargoTargetKind::Bin {
        cmd.arg("--bin").arg(&target.target_name);
    }

    if target.profile != "dev" {
        cmd.arg("--profile").arg(&target.profile);
    }

    if !target.features.is_empty() {
        cmd.arg("--features").arg(target.features.join(","));
    }

    let output = cmd
        .output()
        .await
        .map_err(|e: std::io::Error| CargoError::BuildFailure(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CargoError::BuildFailure(stderr.to_string()));
    }

    Ok(())
}

fn newest_file_mtime(dir: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(t) = newest_file_mtime(&path) {
                    newest = Some(newest.map_or(t, |n| n.max(t)));
                }
            } else if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    newest = Some(newest.map_or(mtime, |n| n.max(mtime)));
                }
            }
        }
    }
    newest
}
