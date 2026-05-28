//! Rust symbol demangling wrapper.

/// Demangle a Rust symbol name.
pub fn demangle(name: &str) -> String {
    rustc_demangle::demangle(name).to_string()
}
