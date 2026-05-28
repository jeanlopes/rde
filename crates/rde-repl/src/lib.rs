//! REPL — Read-Eval-Print Loop for interactive debugging.

use rde_core::{EngineCommand, EngineEvent};
use rde_pretty_print::{FormatBudget, PrettyValue};
use tokio::sync::mpsc;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod executor;
pub mod formatter;
pub mod parser;

/// REPL configuration state.
#[derive(Debug, Clone)]
pub struct ReplConfig {
    pub auto_disassemble: bool,
    pub disassembly_count: usize,
    pub pretty_print: bool,
    pub print_limit: u32,
    pub print_depth: u32,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            auto_disassemble: false,
            disassembly_count: 10,
            pretty_print: true,
            print_limit: 100,
            print_depth: 5,
        }
    }
}

/// Run the REPL loop.
pub async fn run(
    mut event_rx: mpsc::UnboundedReceiver<EngineEvent>,
    command_tx: mpsc::UnboundedSender<EngineCommand>,
) {
    info!("REPL started");
    let config = Arc::new(Mutex::new(ReplConfig::default()));
    let config_events = config.clone();
    let cmd_tx = command_tx.clone();
    let current_pid = Arc::new(Mutex::new(None::<u32>));
    let current_pid_events = current_pid.clone();

    // Spawn event listener
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let auto_disassemble = config_events.lock().await.auto_disassemble;
            let count = config_events.lock().await.disassembly_count;
            let pretty_print = config_events.lock().await.pretty_print;
            let print_limit = config_events.lock().await.print_limit;
            let print_depth = config_events.lock().await.print_depth;
            match event {
                EngineEvent::ProcessLaunched { pid } => {
                    *current_pid_events.lock().await = Some(pid);
                    println!("Processo iniciado: PID {pid}")
                }
                EngineEvent::ProcessAttached { pid } => {
                    *current_pid_events.lock().await = Some(pid);
                    println!("Anexado ao processo: PID {pid}")
                }
                EngineEvent::BreakpointHit { kind, address, thread_id } => {
                    match kind {
                        rde_core::BreakpointKind::SystemInitial => {
                            println!("[System Breakpoint] Hit em 0x{address:x} — Thread {thread_id}")
                        }
                        rde_core::BreakpointKind::UserDefined(id) => {
                            println!("[Breakpoint {id}] Hit em 0x{address:x} — Thread {thread_id}")
                        }
                        rde_core::BreakpointKind::Unknown => {
                            println!("[Unknown Breakpoint] Hit em 0x{address:x} — Thread {thread_id}")
                        }
                    }
                    if auto_disassemble {
                        let _ = cmd_tx.send(EngineCommand::Disassemble {
                            address: None,
                            symbol: None,
                            thread_id: Some(thread_id),
                            count: Some(count),
                        });
                    }
                }
                EngineEvent::SingleStep { address, thread_id } => {
                    println!("[Single Step] 0x{address:x} — Thread {thread_id}");
                    if auto_disassemble {
                        let _ = cmd_tx.send(EngineCommand::Disassemble {
                            address: None,
                            symbol: None,
                            thread_id: Some(thread_id),
                            count: Some(count),
                        });
                    }
                }
                EngineEvent::Exception { code, address } => {
                    println!("[Exception 0x{code:08X}] em 0x{address:x}");
                    if auto_disassemble {
                        let _ = cmd_tx.send(EngineCommand::Disassemble {
                            address: None,
                            symbol: None,
                            thread_id: None,
                            count: Some(count),
                        });
                    }
                }
                EngineEvent::ProcessExited { code } => println!("[Processo encerrado] código: {code}"),
                EngineEvent::ThreadCreated { id, .. } => println!("[Thread criada] TID {id}"),
                EngineEvent::ThreadExited { id, code } => println!("[Thread encerrada] TID {id} (código: {code})"),
                EngineEvent::ModuleLoaded { name, base } => println!("[Módulo carregado] {name} em 0x{base:x}"),
                EngineEvent::ModuleUnloaded { base } => println!("[Módulo descarregado] 0x{base:x}"),
                EngineEvent::Error { message } => println!("Erro: {message}"),
                EngineEvent::Output { message } => println!("{message}"),
                EngineEvent::PrettyValue { value } => {
                    println!("{}", format_pretty_value(&value));
                }
                EngineEvent::TaskList { tasks } => {
                    println!("{}", format_task_list(&tasks));
                }
                EngineEvent::MemoryBytes { address, bytes } => {
                    if pretty_print {
                        let reader = MemoryBytesReader { bytes: bytes.clone() };
                        let mut budget = FormatBudget::new(print_depth, print_limit, 4096);
                        let registry = rde_pretty_print::built_in_registry();
                        // Try Vec first, then String
                        let result = registry.lookup("std::vec::Vec")
                            .and_then(|p| p.format(&reader, 0, &mut budget).ok())
                            .or_else(|| {
                                let mut budget2 = FormatBudget::new(print_depth, print_limit, 4096);
                                registry.lookup("std::string::String")
                                    .and_then(|p| p.format(&reader, 0, &mut budget2).ok())
                            });
                        if let Some(value) = result {
                            println!("Pretty-print @ 0x{address:x}:\n{}", format_pretty_value(&value));
                        } else {
                            println!("Memory @ 0x{address:x}: {} bytes", bytes.len());
                        }
                    } else {
                        println!("Memory @ 0x{address:x}: {} bytes", bytes.len());
                    }
                }
                _ => {
                    // Structured events consumed by TUI
                }
            }
        }
    });

    // REPL input loop
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    use tokio::io::AsyncBufReadExt;

    loop {
        print!("rde> ");
        let _ = std::io::Write::flush(&mut std::io::stdout());

        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                // Local config commands handled before parsing
                if let Some(rest) = line.strip_prefix("set pretty-print ") {
                    let mut cfg = config.lock().await;
                    cfg.pretty_print = match rest.trim() {
                        "on" => true,
                        "off" => false,
                        _ => { println!("usage: set pretty-print on|off"); continue; }
                    };
                    println!("pretty-print = {}", if cfg.pretty_print { "on" } else { "off" });
                    continue;
                }
                if let Some(rest) = line.strip_prefix("set print-limit ") {
                    let l: u32 = match rest.trim().parse() {
                        Ok(v) => v,
                        Err(_) => { println!("usage: set print-limit <number>"); continue; }
                    };
                    config.lock().await.print_limit = l;
                    println!("print-limit = {l}");
                    continue;
                }
                if let Some(rest) = line.strip_prefix("set print-depth ") {
                    let d: u32 = match rest.trim().parse() {
                        Ok(v) => v,
                        Err(_) => { println!("usage: set print-depth <number>"); continue; }
                    };
                    config.lock().await.print_depth = d;
                    println!("print-depth = {d}");
                    continue;
                }
                match parser::parse(line) {
                    Ok(EngineCommand::Quit) => {
                        let _ = command_tx.send(EngineCommand::Quit);
                        break;
                    }
                    Ok(EngineCommand::SetDisassemblyConfig { auto_show, count }) => {
                        {
                            let mut cfg = config.lock().await;
                            if let Some(auto) = auto_show {
                                cfg.auto_disassemble = auto;
                                println!("auto-disassemble = {}", if auto { "on" } else { "off" });
                            }
                            if let Some(c) = count {
                                cfg.disassembly_count = c;
                                println!("disassembly-count = {c}");
                            }
                        }
                        let _ = command_tx.send(EngineCommand::SetDisassemblyConfig { auto_show, count });
                    }
                    Ok(EngineCommand::ListTasks) => {
                        let pid = *current_pid.lock().await;
                        if let Some(pid) = pid {
                            match rde_orchestrator::list_tokio_tasks(pid) {
                                Ok(tasks) => {
                                    println!("{}", format_task_list(&tasks));
                                }
                                Err(e) => println!("Erro: {e}"),
                            }
                        } else {
                            let _ = command_tx.send(EngineCommand::ListTasks);
                        }
                    }
                    Ok(cmd) => {
                        let _ = command_tx.send(cmd);
                    }
                    Err(e) => println!("Erro: {e}"),
                }
            }
            Err(e) => {
                println!("Erro de leitura: {e}");
                break;
            }
        }
    }

    info!("REPL exiting");
}

fn format_pretty_value(value: &PrettyValue) -> String {
    match value {
        PrettyValue::Scalar(s) => s.clone(),
        PrettyValue::Enum { name, payload: None } => name.clone(),
        PrettyValue::Enum { name, payload: Some(p) } => {
            format!("{}({})", name, format_pretty_value(p))
        }
        PrettyValue::Sequence(items) => {
            let elems: Vec<_> = items.iter().map(format_pretty_value).collect();
            format!("[{}]", elems.join(", "))
        }
        PrettyValue::Map(items) => {
            let elems: Vec<_> = items.iter()
                .map(|(k, v)| format!("{}: {}", format_pretty_value(k), format_pretty_value(v)))
                .collect();
            format!("{{{}}}", elems.join(", "))
        }
        PrettyValue::Raw(s) => s.clone(),
        PrettyValue::Truncated => "...".to_string(),
    }
}

fn format_task_list(tasks: &[rde_tokio::task::AsyncTask]) -> String {
    if tasks.is_empty() {
        return "No tasks found.".to_string();
    }
    let mut lines = vec!["ID    State      Function                Thread".to_string()];
    lines.push("----  ---------  ----------------------  ------".to_string());
    for task in tasks {
        let state = format!("{:?}", task.state);
        let name = task.function_name.as_deref().unwrap_or("unknown");
        let tid = task.runtime_thread_id.map(|t| t.to_string()).unwrap_or_else(|| "-".to_string());
        lines.push(format!(
            "{:<4}  {:<9}  {:<22}  {}",
            task.task_id, state, name, tid
        ));
    }
    lines.join("\n")
}

struct MemoryBytesReader {
    bytes: Vec<u8>,
}

impl rde_core::memory::MemoryReader for MemoryBytesReader {
    fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>, rde_core::DebugError> {
        let end = (address + len).min(self.bytes.len());
        if address >= self.bytes.len() {
            return Ok(vec![0u8; len]);
        }
        Ok(self.bytes[address..end].to_vec())
    }
}
