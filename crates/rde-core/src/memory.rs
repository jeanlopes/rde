//! Memory reading abstractions for target process inspection.

use crate::DebugError;

/// Trait for reading bytes from the debuggee's address space.
pub trait MemoryReader: Send + Sync {
    /// Read `len` bytes starting at `address`.
    fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>, DebugError>;
}
