//! Cargo metadata parsing and project resolution.

use crate::CargoError;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Raw cargo metadata output (simplified).
#[derive(Debug, Deserialize)]
pub struct CargoMetadata {
    pub packages: Vec<Package>,
    pub workspace_root: String,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub targets: Vec<Target>,
    pub manifest_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub name: String,
    pub kind: Vec<String>,
}

/// Run `cargo metadata` and parse the JSON output.
pub async fn fetch_metadata(manifest_path: &PathBuf) -> Result<CargoMetadata, CargoError> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(manifest_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e: std::io::Error| CargoError::MetadataFailure(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CargoError::MetadataFailure(stderr.to_string()));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| CargoError::MetadataFailure(e.to_string()))
}
