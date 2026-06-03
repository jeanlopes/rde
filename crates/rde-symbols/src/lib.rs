//! Symbol engine — DbgHelp integration, stack walking, and demangling.

use rde_core::{DebugError, StackFrame, SymbolInfo};

pub mod dbghelp;
pub mod demangler;

pub use dbghelp::DbgHelpSymbolEngine;

/// Trait for symbol resolution backends.
pub trait SymbolEngine: Send + Sync {
    /// Initialize symbol handling for the given process.
    fn initialize(&mut self, process_id: u32, exe_path: Option<&std::path::Path>, base_address: Option<u64>, process_handle: Option<isize>) -> Result<(), DebugError>;

    /// Resolve a symbol by address.
    fn resolve(&self, address: u64) -> Result<SymbolInfo, DebugError>;

    /// Walk the call stack for the given thread.
    fn walk_stack(&self, thread_id: u32) -> Result<Vec<StackFrame>, DebugError>;

    /// Load symbols for a newly loaded module.
    fn load_module(&mut self, base: u64, path: &str) -> Result<(), DebugError>;

    /// Unload symbols for a module that has been unloaded.
    fn unload_module(&mut self, base: u64) -> Result<(), DebugError>;

    /// Resolve symbol(s) by name. Returns all matching addresses.
    fn resolve_by_name(&self, name: &str) -> Result<Vec<u64>, DebugError>;

    /// Cleanup symbol resources.
    fn cleanup(&mut self) -> Result<(), DebugError>;
}

/// Stub symbol engine for MVP development.
pub struct StubSymbolEngine;

impl SymbolEngine for StubSymbolEngine {
    fn initialize(&mut self, _process_id: u32, _exe_path: Option<&std::path::Path>, _base_address: Option<u64>, _process_handle: Option<isize>) -> Result<(), DebugError> {
        Ok(())
    }

    fn resolve(&self, _address: u64) -> Result<SymbolInfo, DebugError> {
        Err(DebugError::Internal("symbol resolution not yet implemented".into()))
    }

    fn walk_stack(&self, _thread_id: u32) -> Result<Vec<StackFrame>, DebugError> {
        Ok(vec![])
    }

    fn load_module(&mut self, _base: u64, _path: &str) -> Result<(), DebugError> {
        Ok(())
    }

    fn unload_module(&mut self, _base: u64) -> Result<(), DebugError> {
        Ok(())
    }

    fn resolve_by_name(&self, _name: &str) -> Result<Vec<u64>, DebugError> {
        Err(DebugError::Internal("symbol resolution not yet implemented".into()))
    }

    fn cleanup(&mut self) -> Result<(), DebugError> {
        Ok(())
    }
}
