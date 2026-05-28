//! Breakpoint hit handling — restore-step-reinstall pattern.

use rde_core::{DebugError, RegisterContext};

/// Restore the original byte, step once, then reinstall the breakpoint.
pub fn handle_hit(
    _original_byte: u8,
    _address: u64,
    _ctx: &mut RegisterContext,
) -> Result<(), DebugError> {
    // TODO: implement restore-step-reinstall logic
    Ok(())
}
