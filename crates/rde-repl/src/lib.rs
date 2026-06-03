//! REPL — Read-Eval-Print Loop for interactive debugging.
//!
//! # Prompt protocol
//! The prompt `rde>` is printed:
//! - After an "info" command completes (engine sends CommandComplete).
//! - After a "pausing" background event (BreakpointHit, StepCompleted, ProcessExited, Exception,
//!   SingleStep from user-initiated step).
//!
//! "Execution" commands (Continue, StepInto, StepOver, StepOut, Launch, Attach) do NOT
//! immediately print the prompt — the process is now running, so the prompt reappears only
//! when it stops again.

use rde_core::{EngineCommand, EngineEvent};
use rde_pretty_print::{FormatBudget, PrettyValue};
use tokio::sync::mpsc;
use tracing::info;

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
    let mut config = ReplConfig::default();
    let mut current_pid: Option<u32> = None;

    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    use tokio::io::AsyncBufReadExt;
    let mut line = String::new();

    loop {
        line.clear();
        tokio::select! {
            // ── User input ────────────────────────────────────────────────────
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    print_prompt();
                    continue;
                }

                // Pre-parse local config shortcuts.
                if let Some(rest) = trimmed.strip_prefix("set pretty-print ") {
                    config.pretty_print = match rest.trim() {
                        "on" => true,
                        "off" => false,
                        _ => {
                            println!("usage: set pretty-print on|off");
                            print_prompt();
                            continue;
                        }
                    };
                    println!("pretty-print = {}", if config.pretty_print { "on" } else { "off" });
                    print_prompt();
                    continue;
                }
                if let Some(rest) = trimmed.strip_prefix("set print-limit ") {
                    match rest.trim().parse::<u32>() {
                        Ok(v) => {
                            config.print_limit = v;
                            println!("print-limit = {v}");
                        }
                        Err(_) => println!("usage: set print-limit <number>"),
                    }
                    print_prompt();
                    continue;
                }
                if let Some(rest) = trimmed.strip_prefix("set print-depth ") {
                    match rest.trim().parse::<u32>() {
                        Ok(v) => {
                            config.print_depth = v;
                            println!("print-depth = {v}");
                        }
                        Err(_) => println!("usage: set print-depth <number>"),
                    }
                    print_prompt();
                    continue;
                }

                match parser::parse(&trimmed) {
                    Err(e) => {
                        println!("Erro: {e}");
                        print_prompt();
                    }
                    Ok(EngineCommand::Quit) => {
                        let _ = command_tx.send(EngineCommand::Quit);
                        break;
                    }
                    Ok(EngineCommand::ListTasks) => {
                        if let Some(pid) = current_pid {
                            match rde_orchestrator::list_tokio_tasks(pid) {
                                Ok(tasks) => println!("{}", format_task_list(&tasks)),
                                Err(e) => println!("Erro: {e}"),
                            }
                        } else {
                            let _ = command_tx.send(EngineCommand::ListTasks);
                            wait_for_complete(&mut event_rx, &config, false).await;
                        }
                        print_prompt();
                    }
                    Ok(EngineCommand::SetDisassemblyConfig { auto_show, count }) => {
                        if let Some(auto) = auto_show {
                            config.auto_disassemble = auto;
                            println!("auto-disassemble = {}", if auto { "on" } else { "off" });
                        }
                        if let Some(c) = count {
                            config.disassembly_count = c;
                            println!("disassembly-count = {c}");
                        }
                        let _ = command_tx.send(EngineCommand::SetDisassemblyConfig { auto_show, count });
                        wait_for_complete(&mut event_rx, &config, false).await;
                        print_prompt();
                    }
                    Ok(cmd) if is_execution_command(&cmd) => {
                        // Execution commands start/resume the process.
                        // The prompt will reappear when the process pauses again.
                        let _ = command_tx.send(cmd);
                        // Drain the CommandComplete that the engine always sends —
                        // we don't want it to mistakenly trigger a prompt later.
                        wait_for_complete(&mut event_rx, &config, true).await;
                        // No print_prompt() here.
                    }
                    Ok(cmd) => {
                        let _ = command_tx.send(cmd);
                        wait_for_complete(&mut event_rx, &config, false).await;
                        print_prompt();
                    }
                }
            }

            // ── Background events from the engine ─────────────────────────────
            Some(event) = event_rx.recv() => {
                let pausing = is_pausing_event(&event);
                let tid = thread_id_of(&event);

                match &event {
                    EngineEvent::ProcessLaunched { pid, .. } | EngineEvent::ProcessAttached { pid } => {
                        current_pid = Some(*pid);
                    }
                    _ => {}
                }

                print_event_stdout(&event, &config);

                if pausing {
                    if config.auto_disassemble {
                        let _ = command_tx.send(EngineCommand::Disassemble {
                            address: None,
                            symbol: None,
                            thread_id: tid,
                            count: Some(config.disassembly_count),
                        });
                        // Wait for disassembly to arrive before showing prompt.
                        wait_for_complete(&mut event_rx, &config, false).await;
                    }
                    print_prompt();
                }
            }
        }
    }

    info!("REPL exiting");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Wait until the engine sends CommandComplete, printing any interim events.
/// For execution commands (`for_execution = true`), a pausing event that arrives
/// before CommandComplete must also trigger the prompt, because the process is
/// now paused and the user needs to see `rde>`.
async fn wait_for_complete(
    event_rx: &mut mpsc::UnboundedReceiver<EngineEvent>,
    config: &ReplConfig,
    for_execution: bool,
) {
    loop {
        match event_rx.recv().await {
            None => return,
            Some(EngineEvent::CommandComplete) => return,
            Some(event) => {
                let pausing = is_pausing_event(&event);
                print_event_stdout(&event, config);
                if for_execution && pausing {
                    if config.auto_disassemble {
                        // We can't easily send Disassemble here without command_tx,
                        // so skip auto-disassemble for events that arrive mid-command.
                    }
                    print_prompt();
                }
            }
        }
    }
}

/// Execution commands start/resume the process — no immediate prompt.
fn is_execution_command(cmd: &EngineCommand) -> bool {
    matches!(
        cmd,
        EngineCommand::Continue
            | EngineCommand::StepInto
            | EngineCommand::StepOver
            | EngineCommand::StepOut
            | EngineCommand::Launch { .. }
            | EngineCommand::Attach { .. }
    )
}

/// These events mean the process has paused — show the prompt afterwards.
fn is_pausing_event(event: &EngineEvent) -> bool {
    matches!(
        event,
        EngineEvent::BreakpointHit { .. }
            | EngineEvent::SingleStep { .. }
            | EngineEvent::StepCompleted { .. }
            | EngineEvent::ProcessExited { .. }
            | EngineEvent::Exception { .. }
    )
}

fn thread_id_of(event: &EngineEvent) -> Option<u32> {
    match event {
        EngineEvent::BreakpointHit { thread_id, .. } => Some(*thread_id),
        EngineEvent::SingleStep { thread_id, .. } => Some(*thread_id),
        EngineEvent::StepCompleted { thread_id, .. } => Some(*thread_id),
        _ => None,
    }
}

fn print_prompt() {
    println!("rde>");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

fn print_event_stdout(event: &EngineEvent, config: &ReplConfig) {
    match event {
        EngineEvent::ProcessLaunched { pid, .. } => println!("Processo iniciado: PID {pid}"),
        EngineEvent::ProcessAttached { pid } => println!("Anexado ao processo: PID {pid}"),
        EngineEvent::BreakpointHit { kind, address, thread_id } => match kind {
            rde_core::BreakpointKind::SystemInitial => {
                println!("[System Breakpoint] Hit em 0x{address:x} — Thread {thread_id}")
            }
            rde_core::BreakpointKind::UserDefined(id) => {
                println!("[Breakpoint {id}] Hit em 0x{address:x} — Thread {thread_id}")
            }
            rde_core::BreakpointKind::Unknown => {
                println!("[Unknown Breakpoint] Hit em 0x{address:x} — Thread {thread_id}")
            }
            rde_core::BreakpointKind::Temporary => {}
        },
        EngineEvent::SingleStep { address, thread_id } => {
            println!("[Single Step] 0x{address:x} — Thread {thread_id}");
        }
        EngineEvent::Exception { code, address } => {
            println!("[Exception 0x{code:08X}] em 0x{address:x}");
        }
        EngineEvent::ProcessExited { code } => println!("[Processo encerrado] código: {code}"),
        EngineEvent::ThreadCreated { id, .. } => println!("[Thread criada] TID {id}"),
        EngineEvent::ThreadExited { id, code } => {
            println!("[Thread encerrada] TID {id} (código: {code})")
        }
        EngineEvent::ModuleLoaded { name, base } => {
            println!("[Módulo carregado] {name} em 0x{base:x}")
        }
        EngineEvent::ModuleUnloaded { base } => println!("[Módulo descarregado] 0x{base:x}"),
        EngineEvent::Error { message } => println!("Erro: {message}"),
        EngineEvent::Output { message } => println!("{message}"),
        EngineEvent::PrettyValue { value } => println!("{}", format_pretty_value(value)),
        EngineEvent::TaskList { tasks } => println!("{}", format_task_list(tasks)),
        EngineEvent::MemoryBytes { address, bytes } => {
            if config.pretty_print {
                let reader = MemoryBytesReader { bytes: bytes.clone() };
                let mut budget = FormatBudget::new(config.print_depth, config.print_limit, 4096);
                let registry = rde_pretty_print::built_in_registry();
                let result = registry
                    .lookup("std::vec::Vec")
                    .and_then(|p| p.format(&reader, 0, &mut budget).ok())
                    .or_else(|| {
                        let mut budget2 =
                            FormatBudget::new(config.print_depth, config.print_limit, 4096);
                        registry
                            .lookup("std::string::String")
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
        EngineEvent::StepCompleted { thread_id, address } => {
            println!("[Step] Instrução executada. IP=0x{address:x} — Thread {thread_id}");
        }
        // Structured events consumed by TUI / already printed above.
        EngineEvent::CommandComplete
        | EngineEvent::Registers { .. }
        | EngineEvent::Disassembly { .. }
        | EngineEvent::BreakpointList { .. }
        | EngineEvent::StackTrace { .. }
        | EngineEvent::CargoLaunchResult { .. } => {}
    }
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
            let elems: Vec<_> = items
                .iter()
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
        let tid = task
            .runtime_thread_id
            .map(|t| t.to_string())
            .unwrap_or_else(|| "-".to_string());
        lines.push(format!("{:<4}  {:<9}  {:<22}  {}", task.task_id, state, name, tid));
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
