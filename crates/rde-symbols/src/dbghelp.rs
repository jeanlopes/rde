//! DbgHelp.dll dynamic loading and wrappers.

use rde_core::DebugError;

/// Dynamic loader for DbgHelp APIs.
pub struct DbgHelpLoader {
    // TODO: libloading handles and function pointers
}

impl DbgHelpLoader {
    pub fn new() -> Result<Self, DebugError> {
        // TODO: LoadLibrary("DbgHelp.dll")
        Ok(Self {})
    }
}
