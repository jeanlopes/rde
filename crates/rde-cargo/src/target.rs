//! Cargo target resolution.

use crate::metadata::{CargoMetadata, Package};
use crate::CargoError;
use rde_core::cargo::{CargoTarget, CargoTargetKind};
use std::path::PathBuf;

/// Resolve a `CargoTarget` from metadata and user-specified options.
pub fn resolve_target(
    metadata: &CargoMetadata,
    package_name: Option<&str>,
    target_name: Option<&str>,
    profile: &str,
    features: &[String],
) -> Result<CargoTarget, CargoError> {
    let pkg = find_package(metadata, package_name)?;
    let tgt = find_target(pkg, target_name)?;

    let artifact_path = build_artifact_path(&metadata.workspace_root, profile, &tgt.name);

    Ok(CargoTarget {
        package_name: pkg.name.clone(),
        target_name: tgt.name.clone(),
        target_kind: parse_target_kind(&tgt.kind),
        profile: profile.to_string(),
        features: features.to_vec(),
        artifact_path,
    })
}

fn find_package<'a>(
    metadata: &'a CargoMetadata,
    name: Option<&str>,
) -> Result<&'a Package, CargoError> {
    match name {
        Some(n) => metadata
            .packages
            .iter()
            .find(|p| p.name == n)
            .ok_or_else(|| CargoError::TargetNotFound(format!("package '{}' not found", n))),
        None => metadata
            .packages
            .first()
            .ok_or_else(|| CargoError::InvalidManifest("no packages in workspace".to_string())),
    }
}

fn find_target<'a>(
    pkg: &'a Package,
    name: Option<&str>,
) -> Result<&'a crate::metadata::Target, CargoError> {
    match name {
        Some(n) => pkg
            .targets
            .iter()
            .find(|t| t.name == n)
            .ok_or_else(|| CargoError::TargetNotFound(format!("target '{}' not found", n))),
        None => pkg
            .targets
            .iter()
            .find(|t| t.kind.contains(&"bin".to_string()))
            .or_else(|| pkg.targets.first())
            .ok_or_else(|| CargoError::TargetNotFound("no targets in package".to_string())),
    }
}

fn build_artifact_path(workspace_root: &str, profile: &str, target_name: &str) -> PathBuf {
    let profile_dir = match profile {
        "dev" => "debug",
        p => p,
    };
    PathBuf::from(workspace_root)
        .join("target")
        .join(profile_dir)
        .join(format!("{}.exe", target_name))
}

fn parse_target_kind(kinds: &[String]) -> CargoTargetKind {
    if kinds.contains(&"bin".to_string()) {
        CargoTargetKind::Bin
    } else if kinds.contains(&"lib".to_string()) {
        CargoTargetKind::Lib
    } else if kinds.contains(&"test".to_string()) {
        CargoTargetKind::Test
    } else if kinds.contains(&"bench".to_string()) {
        CargoTargetKind::Bench
    } else if kinds.contains(&"example".to_string()) {
        CargoTargetKind::Example
    } else {
        CargoTargetKind::Bin
    }
}
