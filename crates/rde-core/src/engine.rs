//! Debug engine orchestrator.

use crate::channel::engine_channels;
use crate::{DebugBackend, DebugError, EngineCommand, EngineEvent, ProcessHandle};
use crate::breakpoint::BreakpointManager;
use tokio::sync::mpsc;
use tracing::{info, instrument};

/// Orchestrates a debug session.
pub struct DebugEngine<B: DebugBackend> {
    backend: B,
    session: Option<Session>,
    breakpoints: BreakpointManager,
    command_rx: mpsc::UnboundedReceiver<EngineCommand>,
    event_tx: mpsc::UnboundedSender<EngineEvent>,
    debug_command_tx: Option<mpsc::UnboundedSender<EngineCommand>>,
}

struct Session {
    handle: ProcessHandle,
}

impl<B: DebugBackend> DebugEngine<B> {
    /// Create a new engine and its communication channels.
    pub fn new(backend: B) -> (Self, mpsc::UnboundedSender<EngineCommand>, mpsc::UnboundedReceiver<EngineEvent>) {
        let (command_tx, event_rx, command_rx, event_tx) = engine_channels();
        let engine = Self {
            backend,
            session: None,
            breakpoints: BreakpointManager::new(),
            command_rx,
            event_tx,
            debug_command_tx: Some(command_tx.clone()),
        };
        (engine, command_tx, event_rx)
    }

    /// Run the engine loop, processing commands and events.
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<(), DebugError> {
        info!("DebugEngine started");
        while let Some(cmd) = self.command_rx.recv().await {
            if let EngineCommand::Quit = cmd {
                break;
            }
            if let Err(e) = self.handle_command(cmd).await {
                let _ = self.event_tx.send(EngineEvent::Error {
                    message: e.to_string(),
                });
            }
        }
        info!("DebugEngine shutting down");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_command(&mut self, cmd: EngineCommand) -> Result<(), DebugError> {
        match cmd {
            EngineCommand::Launch { path, args } => {
                let handle = self.backend.launch(&path, &args).await?;
                let pid = handle.process_id;
                self.start_debug_loop(&handle);
                self.session = Some(Session { handle });
                let _ = self.event_tx.send(EngineEvent::ProcessLaunched { pid });
            }
            EngineCommand::Attach { pid } => {
                let handle = self.backend.attach(pid).await?;
                self.start_debug_loop(&handle);
                self.session = Some(Session { handle });
                let _ = self.event_tx.send(EngineEvent::ProcessAttached { pid });
            }
            EngineCommand::Continue => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                self.backend.continue_execution(&session.handle).await?;
            }
            EngineCommand::StepInto => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                // TODO: get selected thread
                self.backend.single_step(&session.handle, 0).await?;
            }
            EngineCommand::SetBreakpoint { address, .. } => {
                if let Some(addr) = address {
                    self.set_breakpoint(addr).await?;
                } else {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Breakpoint by symbol not yet implemented".into(),
                    });
                }
            }
            EngineCommand::DeleteBreakpoint { id } => {
                self.breakpoints.remove(id)?;
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!("Breakpoint {id} removido."),
                });
            }
            EngineCommand::ReadRegisters { thread_id } => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let tid = thread_id.unwrap_or(0); // TODO: selected thread
                let regs = self.backend.get_registers(&session.handle, tid).await?;
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format_registers(&regs),
                });
            }
            EngineCommand::ReadMemory { address, size } => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let bytes = self.backend.read_memory(&session.handle, address, size).await?;
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format_hex_dump(address, &bytes),
                });
            }
            EngineCommand::WriteMemory { address, bytes } => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                self.backend.write_memory(&session.handle, address, &bytes).await?;
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!("Memória escrita em 0x{address:x} ({} bytes)", bytes.len()),
                });
            }
            EngineCommand::Backtrace { .. } => {
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: "Backtrace not yet implemented".into(),
                });
            }
            EngineCommand::ListThreads => {
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: "Threads not yet implemented".into(),
                });
            }
            EngineCommand::ListModules => {
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: "Modules not yet implemented".into(),
                });
            }
            EngineCommand::Quit => {}
        }
        Ok(())
    }

    fn start_debug_loop(&mut self, handle: &ProcessHandle) {
        if let Some(_tx) = self.debug_command_tx.take() {
            let (debug_tx, debug_rx) = mpsc::unbounded_channel();
            // Replace the old sender with the new one so future commands also go to debug loop
            self.debug_command_tx = Some(debug_tx.clone());
            self.backend.on_session_started(handle, self.event_tx.clone(), debug_rx);
        }
    }

    async fn set_breakpoint(&mut self, address: u64) -> Result<(), DebugError> {
        let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;

        // Read original byte
        let mut original = [0u8; 1];
        let read = self.backend.read_memory(&session.handle, address, 1).await?;
        if read.len() != 1 {
            return Err(DebugError::Win32Error {
                code: 0,
                message: "Failed to read byte for breakpoint".into(),
            });
        }
        original[0] = read[0];

        // Suspend all threads before patching (simplified: main thread only for MVP)
        // TODO: enumerate and suspend all threads

        // Write INT3 (0xCC)
        self.backend.write_memory(&session.handle, address, &[0xCC]).await?;

        let id = self.breakpoints.set(address, original[0]);

        let _ = self.event_tx.send(EngineEvent::Output {
            message: format!("Breakpoint {id} definido em 0x{address:x}"),
        });
        Ok(())
    }
}

fn format_registers(ctx: &crate::RegisterContext) -> String {
    format!(
        "RAX: {:016X}  RBX: {:016X}\n\
         RCX: {:016X}  RDX: {:016X}\n\
         RSI: {:016X}  RDI: {:016X}\n\
         RBP: {:016X}  RSP: {:016X}\n\
         RIP: {:016X}  RFLAGS: {:016X}\n\
         R8:  {:016X}  R9:  {:016X}\n\
         R10: {:016X}  R11: {:016X}\n\
         R12: {:016X}  R13: {:016X}\n\
         R14: {:016X}  R15: {:016X}",
        ctx.rax, ctx.rbx, ctx.rcx, ctx.rdx, ctx.rsi, ctx.rdi, ctx.rbp, ctx.rsp,
        ctx.rip, ctx.rflags, ctx.r8, ctx.r9, ctx.r10, ctx.r11, ctx.r12, ctx.r13,
        ctx.r14, ctx.r15
    )
}

fn format_hex_dump(address: u64, bytes: &[u8]) -> String {
    let mut lines = Vec::new();
    for chunk in bytes.chunks(16) {
        let hex: String = chunk.iter().map(|b| format!("{:02X} ", b)).collect();
        let ascii: String = chunk
            .iter()
            .map(|b| if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' })
            .collect();
        lines.push(format!("0x{:016X}: {:48} |{ascii}|", address, hex));
    }
    lines.join("\n")
}
