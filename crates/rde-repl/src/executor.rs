//! Command executor — dispatches parsed commands to the engine.

use rde_core::EngineCommand;

/// Execute a command and return a human-readable result.
pub fn execute(cmd: &EngineCommand) -> String {
    match cmd {
        EngineCommand::Launch { path, .. } => format!("Launching: {}", path.display()),
        EngineCommand::Attach { pid } => format!("Attaching to PID: {pid}"),
        EngineCommand::Continue => "Continuing execution...".into(),
        EngineCommand::StepInto => "Stepping...".into(),
        EngineCommand::SetBreakpoint { address, symbol } => {
            if let Some(addr) = address {
                format!("Setting breakpoint at 0x{addr:x}")
            } else if let Some(sym) = symbol {
                format!("Setting breakpoint at {sym}")
            } else {
                "Invalid breakpoint".into()
            }
        }
        EngineCommand::DeleteBreakpoint { id } => format!("Deleting breakpoint {id}"),
        EngineCommand::ReadRegisters { .. } => "Reading registers...".into(),
        EngineCommand::ReadMemory { address, size } => {
            format!("Reading {size} bytes from 0x{address:x}")
        }
        EngineCommand::WriteMemory { address, bytes } => {
            format!("Writing {} bytes to 0x{address:x}", bytes.len())
        }
        EngineCommand::Backtrace { .. } => "Generating backtrace...".into(),
        EngineCommand::ListThreads => "Listing threads...".into(),
        EngineCommand::ListModules => "Listing modules...".into(),
        EngineCommand::Quit => "Quitting...".into(),
    }
}
