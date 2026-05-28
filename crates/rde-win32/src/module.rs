//! Module (DLL) enumeration and tracking.

use rde_core::Module;

/// Enumerate loaded modules in the target process.
pub fn enumerate_modules() -> Vec<Module> {
    // TODO: implement module enumeration via debug events
    vec![]
}
