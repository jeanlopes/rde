//! Cargo project and target types shared across crates.

use std::path::PathBuf;

/// Kind of Cargo target.
#[derive(Debug, Clone, PartialEq)]
pub enum CargoTargetKind {
    Bin,
    Lib,
    Test,
    Bench,
    Example,
}

/// A resolved Cargo build target.
#[derive(Debug, Clone)]
pub struct CargoTarget {
    /// Package containing the target.
    pub package_name: String,
    /// Name of the target.
    pub target_name: String,
    /// Kind of target.
    pub target_kind: CargoTargetKind,
    /// Build profile (e.g., `dev`, `release`).
    pub profile: String,
    /// Active Cargo features.
    pub features: Vec<String>,
    /// Resolved path to the compiled executable.
    pub artifact_path: PathBuf,
}

/// Cargo launch request parameters.
#[derive(Debug, Clone)]
pub struct CargoLaunchRequest {
    /// Path to the `Cargo.toml` manifest.
    pub manifest_path: PathBuf,
    /// Specific package to build (for workspaces).
    pub package: Option<String>,
    /// Specific target to build.
    pub target: Option<String>,
    /// Build profile.
    pub profile: String,
    /// Features to enable.
    pub features: Vec<String>,
}
