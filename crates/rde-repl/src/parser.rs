//! Hand-written command parser for the REPL.

use rde_core::EngineCommand;
use std::path::PathBuf;

/// Parse a REPL command string.
pub fn parse(input: &str) -> Result<EngineCommand, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Err("empty command".into());
    }

    match parts[0] {
        "launch" => {
            if parts.len() < 2 {
                return Err("usage: launch <path> [args...]".into());
            }
            Ok(EngineCommand::Launch {
                path: PathBuf::from(parts[1]),
                args: parts[2..].iter().map(|s| s.to_string()).collect(),
            })
        }
        "attach" => {
            if parts.len() < 2 {
                return Err("usage: attach <pid>".into());
            }
            let pid = parts[1].parse().map_err(|_| "invalid pid")?;
            Ok(EngineCommand::Attach { pid })
        }
        "continue" | "c" => Ok(EngineCommand::Continue),
        "step" | "s" => Ok(EngineCommand::StepInto),
        "break" => {
            if parts.len() < 2 {
                return Err("usage: break <address|symbol>".into());
            }
            let arg = parts[1];
            if let Ok(addr) = parse_hex(arg) {
                Ok(EngineCommand::SetBreakpoint {
                    address: Some(addr),
                    symbol: None,
                })
            } else {
                Ok(EngineCommand::SetBreakpoint {
                    address: None,
                    symbol: Some(arg.to_string()),
                })
            }
        }
        "delbreak" => {
            if parts.len() < 2 {
                return Err("usage: delbreak <id>".into());
            }
            let id = parts[1].parse().map_err(|_| "invalid breakpoint id")?;
            Ok(EngineCommand::DeleteBreakpoint { id })
        }
        "regs" | "registers" => Ok(EngineCommand::ReadRegisters { thread_id: None }),
        "x" => {
            if parts.len() < 2 {
                return Err("usage: x <address> [size]".into());
            }
            let address = parse_hex(parts[1])?;
            let size = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(16);
            Ok(EngineCommand::ReadMemory { address, size })
        }
        "setmem" => {
            if parts.len() < 3 {
                return Err("usage: setmem <address> <byte>...".into());
            }
            let address = parse_hex(parts[1])?;
            let mut bytes = Vec::new();
            for b in &parts[2..] {
                bytes.push(u8::from_str_radix(b, 16).map_err(|_| "invalid byte")?);
            }
            Ok(EngineCommand::WriteMemory { address, bytes })
        }
        "bt" | "backtrace" => Ok(EngineCommand::Backtrace { thread_id: None }),
        "threads" | "info" if parts.get(1) == Some(&"threads") => Ok(EngineCommand::ListThreads),
        "modules" | "info" if parts.get(1) == Some(&"modules") => Ok(EngineCommand::ListModules),
        "quit" | "q" | "exit" => Ok(EngineCommand::Quit),
        _ => Err(format!("comando desconhecido: {}", parts[0])),
    }
}

fn parse_hex(s: &str) -> Result<u64, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).map_err(|_| format!("invalid hex address: {s}"))
}
