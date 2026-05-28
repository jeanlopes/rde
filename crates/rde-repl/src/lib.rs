//! REPL — Read-Eval-Print Loop for interactive debugging.

use rde_core::{EngineCommand, EngineEvent};
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
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            auto_disassemble: false,
            disassembly_count: 10,
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

    // Spawn event listener
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let auto_disassemble = config_events.lock().await.auto_disassemble;
            let count = config_events.lock().await.disassembly_count;
            match event {
                EngineEvent::ProcessLaunched { pid } => println!("Processo iniciado: PID {pid}"),
                EngineEvent::ProcessAttached { pid } => println!("Anexado ao processo: PID {pid}"),
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
                        // No specific thread_id for exception; let engine use selected/fallback
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
