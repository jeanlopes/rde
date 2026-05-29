//! Integration test: Cargo target resolution from mock metadata.

use rde_cargo::target::resolve_target;
use rde_cargo::metadata::{CargoMetadata, Package, Target};
use rde_core::cargo::CargoTargetKind;

#[test]
fn test_resolve_default_binary_target() {
    let metadata = CargoMetadata {
        packages: vec![Package {
            name: "my-app".to_string(),
            targets: vec![
                Target {
                    name: "my-app".to_string(),
                    kind: vec!["bin".to_string()],
                },
            ],
            manifest_path: "/tmp/my-app/Cargo.toml".to_string(),
        }],
        workspace_root: "/tmp/my-app".to_string(),
    };

    let target = resolve_target(&metadata, None, None, "dev", &[]).unwrap();
    assert_eq!(target.package_name, "my-app");
    assert_eq!(target.target_name, "my-app");
    assert_eq!(target.target_kind, CargoTargetKind::Bin);
    assert!(target.artifact_path.to_string_lossy().contains("my-app.exe"));
}
