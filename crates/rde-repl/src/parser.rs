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
        "threads" => Ok(EngineCommand::ListThreads),
        "info" if parts.get(1) == Some(&"threads") => Ok(EngineCommand::ListThreads),
        "modules" => Ok(EngineCommand::ListModules),
        "info" if parts.get(1) == Some(&"modules") => Ok(EngineCommand::ListModules),
        "print" => {
            if parts.len() < 2 {
                return Err("usage: print <expression>".into());
            }
            Ok(EngineCommand::Print {
                frame_id: 0,
                expression: parts[1..].join(" "),
            })
        }
        "vars" => Ok(EngineCommand::Print {
            frame_id: 0,
            expression: "*".to_string(),
        }),
        "tasks" => Ok(EngineCommand::ListTasks),
        "thread" => {
            if parts.len() < 2 {
                return Err("usage: thread <id>".into());
            }
            let id = parts[1].parse().map_err(|_| "invalid thread id")?;
            Ok(EngineCommand::SelectThread { id })
        }
        "disas" | "disassemble" => {
            let mut address = None;
            let mut symbol = None;
            let mut count = None;
            if parts.len() >= 2 {
                let arg = parts[1];
                if let Ok(addr) = parse_hex(arg) {
                    address = Some(addr);
                } else {
                    symbol = Some(arg.to_string());
                }
                if parts.len() >= 3 {
                    count = parts[2].parse().ok();
                }
            }
            Ok(EngineCommand::Disassemble { address, symbol, thread_id: None, count })
        }
        "set" => {
            if parts.len() < 3 {
                return Err("usage: set <option> <value>".into());
            }
            match parts[1] {
                "auto-disassemble" => {
                    let auto_show = match parts[2] {
                        "on" => Some(true),
                        "off" => Some(false),
                        _ => return Err("usage: set auto-disassemble on|off".into()),
                    };
                    Ok(EngineCommand::SetDisassemblyConfig { auto_show, count: None })
                }
                "disassembly-count" => {
                    let c = parts[2].parse().map_err(|_| "invalid count")?;
                    if c < 1 || c > 100 {
                        return Err("count must be between 1 and 100".into());
                    }
                    Ok(EngineCommand::SetDisassemblyConfig { auto_show: None, count: Some(c) })
                }
                _ => Err(format!("unknown set option: {}", parts[1])),
            }
        }
        "quit" | "q" | "exit" => Ok(EngineCommand::Quit),
        _ => Err(format!("comando desconhecido: {}", parts[0])),
    }
}

fn parse_hex(s: &str) -> Result<u64, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).map_err(|_| format!("invalid hex address: {s}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_threads() {
        assert!(matches!(parse("threads").unwrap(), EngineCommand::ListThreads));
    }

    #[test]
    fn test_parse_thread_id() {
        assert!(matches!(parse("thread 1234").unwrap(), EngineCommand::SelectThread { id: 1234 }));
    }

    #[test]
    fn test_parse_modules() {
        assert!(matches!(parse("modules").unwrap(), EngineCommand::ListModules));
    }

    #[test]
    fn test_parse_disassemble_no_args() {
        let cmd = parse("disassemble").unwrap();
        assert!(matches!(cmd, EngineCommand::Disassemble { address: None, symbol: None, thread_id: None, count: None }));
    }

    #[test]
    fn test_parse_disas_address() {
        let cmd = parse("disas 0x140001000").unwrap();
        assert!(matches!(cmd, EngineCommand::Disassemble { address: Some(0x140001000), symbol: None, thread_id: None, count: None }));
    }

    #[test]
    fn test_parse_disassemble_address_with_count() {
        let cmd = parse("disassemble 0x140001000 20").unwrap();
        assert!(matches!(cmd, EngineCommand::Disassemble { address: Some(0x140001000), symbol: None, thread_id: None, count: Some(20) }));
    }

    #[test]
    fn test_parse_disassemble_symbol() {
        let cmd = parse("disassemble main").unwrap();
        assert!(matches!(cmd, EngineCommand::Disassemble { address: None, symbol: Some(ref s), thread_id: None, count: None } if s == "main"));
    }

    #[test]
    fn test_parse_set_auto_disassemble_on() {
        let cmd = parse("set auto-disassemble on").unwrap();
        assert!(matches!(cmd, EngineCommand::SetDisassemblyConfig { auto_show: Some(true), count: None }));
    }

    #[test]
    fn test_parse_set_auto_disassemble_off() {
        let cmd = parse("set auto-disassemble off").unwrap();
        assert!(matches!(cmd, EngineCommand::SetDisassemblyConfig { auto_show: Some(false), count: None }));
    }

    #[test]
    fn test_parse_set_disassembly_count() {
        let cmd = parse("set disassembly-count 25").unwrap();
        assert!(matches!(cmd, EngineCommand::SetDisassemblyConfig { auto_show: None, count: Some(25) }));
    }
}
