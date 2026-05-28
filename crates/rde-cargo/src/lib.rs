//! RDE Cargo — Cargo project integration for automatic target resolution and build.

pub mod build;
pub mod metadata;
pub mod target;

pub use target::resolve_target;
pub use metadata::fetch_metadata;
pub use build::{is_stale, run_build};

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CargoError {
    #[error("Failed to run cargo metadata: {0}")]
    MetadataFailure(String),
    #[error("Cargo build failed: {0}")]
    BuildFailure(String),
    #[error("Target not found: {0}")]
    TargetNotFound(String),
    #[error("Invalid Cargo.toml manifest: {0}")]
    InvalidManifest(String),
    #[error("Cargo executable not found in PATH")]
    CargoNotFound,
}
