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
        EngineCommand::ReadMemory { address, size, expression } => {
            if let Some(expr) = expression {
                format!("Reading {size} bytes from {expr}")
            } else {
                format!("Reading {size} bytes from 0x{address:x}")
            }
        }
        EngineCommand::WriteMemory { address, bytes } => {
            format!("Writing {} bytes to 0x{address:x}", bytes.len())
        }
        EngineCommand::Backtrace { .. } => "Generating backtrace...".into(),
        EngineCommand::ListThreads => "Listing threads...".into(),
        EngineCommand::ListModules => "Listing modules...".into(),
        EngineCommand::SelectThread { id } => format!("Selecting thread {id}..."),
        EngineCommand::Disassemble { address, symbol, .. } => {
            if let Some(addr) = address {
                format!("Disassembling at 0x{addr:x}...")
            } else if let Some(sym) = symbol {
                format!("Disassembling symbol {sym}...")
            } else {
                "Disassembling at RIP...".into()
            }
        }
        EngineCommand::SetDisassemblyConfig { auto_show, count } => {
            if let Some(auto) = auto_show {
                format!("auto-disassemble = {}", if *auto { "on" } else { "off" })
            } else if let Some(c) = count {
                format!("disassembly-count = {c}")
            } else {
                "No config change".into()
            }
        }
        EngineCommand::Print { frame_id, expression } => {
            format!("Printing {expression} @ frame {frame_id}...")
        }
        EngineCommand::ListTasks => "Listing Tokio tasks...".into(),
        EngineCommand::CargoLaunch { manifest_path, .. } => {
            format!("Launching Cargo project: {}", manifest_path.display())
        }
        EngineCommand::StepOver => "Avançando uma instrução (next)...".into(),
        EngineCommand::StepOut => "Executando até o retorno (finish)...".into(),
        EngineCommand::SetPrintConfig { limit, depth, pretty } => {
            let mut parts = Vec::new();
            if let Some(l) = limit { parts.push(format!("limit={l}")); }
            if let Some(d) = depth { parts.push(format!("depth={d}")); }
            if let Some(p) = pretty { parts.push(format!("pretty={p}")); }
            format!("Print config: {}", parts.join(", "))
        }
        EngineCommand::Quit => "Quitting...".into(),
    }
}
