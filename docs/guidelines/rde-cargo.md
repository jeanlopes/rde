# rde-cargo

Cargo project integration for automatic target resolution and build triggering.

---

## Overview

`rde-cargo` eliminates the friction of manually finding compiled binaries in `target/debug/`. It:
1. Reads `Cargo.toml` metadata
2. Resolves the correct target binary path
3. Triggers `cargo build` when the artifact is stale

## API

### fetch_metadata

```rust
use rde_cargo::metadata::fetch_metadata;
use std::path::PathBuf;

let manifest = PathBuf::from("./Cargo.toml");
let metadata = fetch_metadata(&manifest).await?;
```

Returns parsed `cargo metadata --format-version 1` output.

### resolve_target

```rust
use rde_cargo::target::resolve_target;
use rde_core::cargo::CargoTargetKind;

let target = resolve_target(
    &metadata,
    Some("my-package"),     // package name (None for default)
    Some("my-bin"),         // target name (None for default bin)
    "dev",                  // profile
    &["tokio/full".to_string()], // features
)?;

println!("Artifact: {}", target.artifact_path.display());
println!("Kind: {:?}", target.target_kind); // CargoTargetKind::Bin
```

### is_stale

```rust
use rde_cargo::build::is_stale;
use std::path::Path;

let src_dir = Path::new("./src");
if is_stale(&target, &src_dir) {
    println!("Artifact is stale, build needed");
}
```

Heuristic: compares artifact mtime against newest source file mtime in `src/`.

### run_build

```rust
use rde_cargo::build::run_build;

run_build(&target, &manifest_path).await?;
```

Spawns `cargo build` with the correct package, target, profile, and features.

## Error Types

```rust
pub enum CargoError {
    MetadataFailure(String),
    BuildFailure(String),
    TargetNotFound(String),
    InvalidManifest(String),
    CargoNotFound,
}
```

## Data Types

### CargoMetadata

```rust
pub struct CargoMetadata {
    pub packages: Vec<Package>,
    pub workspace_root: String,
}

pub struct Package {
    pub name: String,
    pub targets: Vec<Target>,
    pub manifest_path: String,
}

pub struct Target {
    pub name: String,
    pub kind: Vec<String>,
}
```

### CargoTarget

```rust
pub struct CargoTarget {
    pub package_name: String,
    pub target_name: String,
    pub target_kind: CargoTargetKind,
    pub profile: String,
    pub features: Vec<String>,
    pub artifact_path: PathBuf,
}

pub enum CargoTargetKind {
    Bin,
    Lib,
    Test,
    Bench,
    Example,
}
```

## Usage Examples

### Basic Resolution

```rust
use rde_cargo::{fetch_metadata, resolve_target};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = PathBuf::from("./Cargo.toml");
    let metadata = fetch_metadata(&manifest).await?;
    let target = resolve_target(&metadata, None, None, "dev", &[])?;
    
    println!("Debug binary: {}", target.artifact_path.display());
    Ok(())
}
```

### Build if Stale

```rust
use rde_cargo::{fetch_metadata, resolve_target, is_stale, run_build};
use std::path::Path;

let metadata = fetch_metadata(&manifest).await?;
let target = resolve_target(&metadata, None, None, "dev", &[])?;

let src_dir = Path::new("src");
if is_stale(&target, &src_dir) {
    println!("Building...");
    run_build(&target, &manifest).await?;
}
```

### Workspace Support

```rust
let target = resolve_target(
    &metadata,
    Some("server"),      // Package in workspace
    Some("api"),         // Specific binary target
    "release",
    &["enterprise".to_string()],
)?;
```

## CLI Integration

The `rde-cli` uses `rde-orchestrator::cargo_resolve_and_build` internally:

```bash
rde-cli cargo debug --package my-crate --bin my-bin --release
```

This calls:
1. `fetch_metadata`
2. `resolve_target`
3. `is_stale` check
4. `run_build` if needed
5. `EngineCommand::Launch` with resolved artifact path

## Testing

### Integration Test

```rust
use rde_cargo::target::resolve_target;
use rde_cargo::metadata::{CargoMetadata, Package, Target};
use rde_core::cargo::CargoTargetKind;

#[test]
fn test_resolve_default_binary() {
    let metadata = CargoMetadata {
        packages: vec![Package {
            name: "my-app".to_string(),
            targets: vec![Target {
                name: "my-app".to_string(),
                kind: vec!["bin".to_string()],
            }],
            manifest_path: "/tmp/Cargo.toml".to_string(),
        }],
        workspace_root: "/tmp".to_string(),
    };

    let target = resolve_target(&metadata, None, None, "dev", &[]).unwrap();
    assert_eq!(target.target_kind, CargoTargetKind::Bin);
    assert!(target.artifact_path.to_string_lossy().contains("my-app.exe"));
}
```

## Artifact Path Convention

For a package `my-app` with profile `dev`:

```
<workspace-root>/target/debug/my-app.exe
```

For profile `release`:

```
<workspace-root>/target/release/my-app.exe
```

For custom profiles:

```
<workspace-root>/target/<profile-name>/my-app.exe
```
