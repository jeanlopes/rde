//! REPL — Read-Eval-Print Loop for interactive debugging.

use rde_core::{EngineCommand, EngineEvent};
use tokio::sync::mpsc;
use tracing::info;

pub mod executor;
pub mod formatter;
pub mod parser;

/// Run the REPL loop.
pub async fn run(
    mut event_rx: mpsc::UnboundedReceiver<EngineEvent>,
    command_tx: mpsc::UnboundedSender<EngineCommand>,
) {
    info!("REPL started");

    // Spawn event listener
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
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
                }
                EngineEvent::ProcessExited { code } => println!("[Processo encerrado] código: {code}"),
                EngineEvent::ThreadCreated { id, .. } => println!("[Thread criada] TID {id}"),
                EngineEvent::ThreadExited { id, code } => println!("[Thread encerrada] TID {id} (código: {code})"),
                EngineEvent::ModuleLoaded { name, base } => println!("[Módulo carregado] {name} em 0x{base:x}"),
                EngineEvent::ModuleUnloaded { base } => println!("[Módulo descarregado] 0x{base:x}"),
                EngineEvent::Error { message } => println!("Erro: {message}"),
                EngineEvent::Output { message } => println!("{message}"),
                _ => {}
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
