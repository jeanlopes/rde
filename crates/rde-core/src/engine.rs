//! Debug engine orchestrator.

use crate::channel::engine_channels;
use crate::disasm::{Disassembler, DisassemblyConfig};
use crate::events::{BreakpointKind, DebugLoopCommand};
use crate::{DebugBackend, DebugError, EngineCommand, EngineEvent, ProcessHandle, RawHandle, Thread, ThreadState, Module};
use crate::breakpoint::BreakpointManager;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, instrument};

/// Snapshot of engine state for structured transition logging.
#[derive(Debug, Clone)]
struct EngineStateSnapshot {
    has_session: bool,
    breakpoint_count: usize,
    stepping_over: Option<u64>,
}

impl<B: DebugBackend> DebugEngine<B> {
    fn current_state(&self) -> EngineStateSnapshot {
        EngineStateSnapshot {
            has_session: self.session.is_some(),
            breakpoint_count: self.breakpoints.list().len(),
            stepping_over: self.stepping_over_breakpoint,
        }
    }

    fn log_transition(&self, from: &EngineStateSnapshot, to: &EngineStateSnapshot) {
        info!(
            target: "rde::engine::transition",
            ?from,
            ?to,
            "state transition"
        );
    }
}

/// Orchestrates a debug session.
pub struct DebugEngine<B: DebugBackend> {
    backend: B,
    session: Option<Session>,
    breakpoints: BreakpointManager,
    command_rx: mpsc::UnboundedReceiver<EngineCommand>,
    event_tx: mpsc::UnboundedSender<EngineEvent>,
    debug_event_rx: mpsc::UnboundedReceiver<EngineEvent>,
    debug_loop_tx: Option<mpsc::UnboundedSender<DebugLoopCommand>>,
    stepping_over_breakpoint: Option<u64>,
    disasm_config: DisassemblyConfig,
}

struct Session {
    handle: ProcessHandle,
    threads: HashMap<u32, Thread>,
    modules: HashMap<u64, Module>,
    selected_thread: Option<u32>,
}

impl<B: DebugBackend> DebugEngine<B> {
    /// Create a new engine and its communication channels.
    pub fn new(backend: B) -> (Self, mpsc::UnboundedSender<EngineCommand>, mpsc::UnboundedReceiver<EngineEvent>) {
        let (command_tx, event_rx, command_rx, event_tx) = engine_channels();
        let (_, debug_event_rx) = mpsc::unbounded_channel();
        let engine = Self {
            backend,
            session: None,
            breakpoints: BreakpointManager::new(),
            command_rx,
            event_tx,
            debug_event_rx,
            debug_loop_tx: None,
            stepping_over_breakpoint: None,
            disasm_config: DisassemblyConfig::default(),
        };
        (engine, command_tx, event_rx)
    }

    /// Run the engine loop, processing commands and events.
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<(), DebugError> {
        info!("DebugEngine started");
        loop {
            tokio::select! {
                Some(cmd) = self.command_rx.recv() => {
                    tracing::debug!(target: "rde::engine", "Received command: {:?}", cmd);
                    if let EngineCommand::Quit = cmd {
                        break;
                    }
                    if let Err(e) = self.handle_command(cmd).await {
                        let _ = self.event_tx.send(EngineEvent::Error {
                            message: e.to_string(),
                        });
                    }
                }
                Some(evt) = self.debug_event_rx.recv() => {
                    tracing::debug!(target: "rde::engine", "Received debug event: {:?}", evt);
                    if let Err(e) = self.handle_event(evt).await {
                        let _ = self.event_tx.send(EngineEvent::Error {
                            message: e.to_string(),
                        });
                    }
                }
                else => {
                    tracing::info!(target: "rde::engine", "All channels closed, exiting");
                    break;
                }
            }
        }
        info!("DebugEngine shutting down");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_command(&mut self, cmd: EngineCommand) -> Result<(), DebugError> {
        match cmd {
            EngineCommand::Launch { path, args } => {
                let (handle, (event_rx, debug_loop_tx)) = self.backend.launch(&path, &args).await?;
                let pid = handle.process_id;
                self.debug_event_rx = event_rx;
                self.debug_loop_tx = Some(debug_loop_tx);
                self.session = Some(Session {
                    handle,
                    threads: HashMap::new(),
                    modules: HashMap::new(),
                    selected_thread: None,
                });
                let _ = self.event_tx.send(EngineEvent::ProcessLaunched { pid });
            }
            EngineCommand::Attach { pid } => {
                let (handle, (event_rx, debug_loop_tx)) = self.backend.attach(pid).await?;
                self.debug_event_rx = event_rx;
                self.debug_loop_tx = Some(debug_loop_tx);
                self.session = Some(Session {
                    handle,
                    threads: HashMap::new(),
                    modules: HashMap::new(),
                    selected_thread: None,
                });
                let _ = self.event_tx.send(EngineEvent::ProcessAttached { pid });
            }
            EngineCommand::Continue => {
                // Forward continue to the debug loop
                if let Some(tx) = &self.debug_loop_tx {
                    let _ = tx.send(DebugLoopCommand::Continue);
                }
            }
            EngineCommand::StepInto => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let tid = session.selected_thread.ok_or(DebugError::Internal(
                    "No thread selected".into()
                ))?;
                self.backend.single_step(&session.handle, tid).await?;
                if let Some(tx) = &self.debug_loop_tx {
                    let _ = tx.send(DebugLoopCommand::Continue);
                }
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
                let list: Vec<_> = self.breakpoints.list().into_iter().cloned().collect();
                let _ = self.event_tx.send(EngineEvent::BreakpointList { list: list.clone() });
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!("Breakpoint {id} removido."),
                });
            }
            EngineCommand::ReadRegisters { thread_id } => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let tid = thread_id.or(session.selected_thread).ok_or(DebugError::Internal(
                    "No thread selected".into()
                ))?;
                let regs = self.backend.get_registers(&session.handle, tid).await?;
                let _ = self.event_tx.send(EngineEvent::Registers {
                    ctx: regs.clone(),
                });
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
            EngineCommand::Backtrace { thread_id } => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let tid = thread_id.or(session.selected_thread).ok_or(DebugError::Internal(
                    "No thread selected".into()
                ))?;
                let ctx = self.backend.get_registers(&session.handle, tid).await?;
                let _ = self.event_tx.send(EngineEvent::Registers {
                    ctx: ctx.clone(),
                });
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!("Backtrace for thread {tid} not yet implemented"),
                });
            }
            EngineCommand::ListThreads => {
                info!(target: "rde::engine::command", "ListThreads");
                if let Some(session) = &self.session {
                    let mut lines = vec![" ID       Estado      Selecionada".to_string()];
                    for (id, thread) in &session.threads {
                        let state = match thread.state {
                            ThreadState::Running => "Running",
                            ThreadState::Suspended => "Suspended",
                            ThreadState::Exited { .. } => "Exited",
                        };
                        let sel = if session.selected_thread == Some(*id) { " *" } else { "" };
                        lines.push(format!(" {:<8} {:<11} {}", id, state, sel));
                    }
                    let _ = self.event_tx.send(EngineEvent::Output {
                        message: lines.join("\n"),
                    });
                } else {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "No active debug session".into(),
                    });
                }
            }
            EngineCommand::ListModules => {
                info!(target: "rde::engine::command", "ListModules");
                if let Some(session) = &self.session {
                    let mut lines = vec![" Nome                Base              Tamanho    Símbolos".to_string()];
                    for module in session.modules.values() {
                        let sym = if module.symbols_loaded { "✓" } else { "✗" };
                        lines.push(format!(
                            " {:<19} 0x{:016X} 0x{:08X} {}",
                            module.name, module.base_address, module.size, sym
                        ));
                    }
                    let _ = self.event_tx.send(EngineEvent::Output {
                        message: lines.join("\n"),
                    });
                } else {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "No active debug session".into(),
                    });
                }
            }
            EngineCommand::SelectThread { id } => {
                info!(target: "rde::engine::command", thread_id = id, "SelectThread");
                if let Some(session) = &mut self.session {
                    let valid = session.threads.get(&id).map_or(false, |t| {
                        !matches!(t.state, ThreadState::Exited { .. })
                    });
                    if valid {
                        session.selected_thread = Some(id);
                        let _ = self.event_tx.send(EngineEvent::Output {
                            message: format!("Thread {id} selecionada."),
                        });
                    } else {
                        let _ = self.event_tx.send(EngineEvent::Error {
                            message: format!("Thread {id} não existe ou já foi encerrada."),
                        });
                    }
                } else {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "No active debug session".into(),
                    });
                }
            }
            EngineCommand::Disassemble { address, symbol, thread_id, count } => {
                info!(target: "rde::engine::command", ?address, ?symbol, ?thread_id, ?count, "Disassemble");
                if let Err(e) = self.handle_disassemble(address, symbol, thread_id, count).await {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: e.to_string(),
                    });
                }
            }
            EngineCommand::SetDisassemblyConfig { auto_show, count } => {
                info!(target: "rde::engine::command", ?auto_show, ?count, "SetDisassemblyConfig");
                if let Some(auto) = auto_show {
                    self.disasm_config.auto_show = auto;
                }
                if let Some(c) = count {
                    self.disasm_config.count = c;
                }
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!(
                        "Disassembly config: count={}, auto_show={}",
                        self.disasm_config.count, self.disasm_config.auto_show
                    ),
                });
            }
            EngineCommand::Quit => {}
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_event(&mut self, evt: EngineEvent) -> Result<(), DebugError> {
        tracing::debug!(target: "rde::engine", "handle_event: {:?}", evt);
        let from_state = self.current_state();

        match evt {
            EngineEvent::BreakpointHit { kind, address, thread_id } => {
                // INVARIANT: System breakpoints (e.g., Windows initial breakpoint in ntdll)
                // must be continued with DBG_CONTINUE only. Never restore original byte,
                // never decrement RIP, never set Trap Flag.
                // Violation: infinite breakpoint→single-step→breakpoint loop.
                let kind = match kind {
                    BreakpointKind::Unknown => {
                        // Resolve from breakpoint manager by address
                        if let Some(bp) = self.breakpoints.find_by_address(address) {
                            BreakpointKind::UserDefined(bp.id)
                        } else {
                            BreakpointKind::SystemInitial
                        }
                    }
                    other => other,
                };

                match kind {
                    BreakpointKind::SystemInitial => {
                        info!(
                            target: "rde::engine::transition",
                            address = format!("0x{:x}", address),
                            thread_id,
                            "SystemInitial breakpoint hit — waiting for user continue"
                        );
                        let _ = self.event_tx.send(EngineEvent::BreakpointHit {
                            kind: BreakpointKind::SystemInitial,
                            address,
                            thread_id,
                        });
                        // Do NOT auto-continue. The debugger stops at the system breakpoint
                        // so the user can inspect state and set breakpoints before running.
                        let to_state = self.current_state();
                        self.log_transition(&from_state, &to_state);
                        return Ok(());
                    }
                    BreakpointKind::UserDefined(id) => {
                        info!(
                            target: "rde::engine::transition",
                            breakpoint_id = id,
                            address = format!("0x{:x}", address),
                            thread_id,
                            "UserDefined breakpoint hit — applying full protocol"
                        );

                        if let Some(session) = &self.session {
                            // 1. Restore original byte
                            if let Some(bp) = self.breakpoints.get(id) {
                                let original = bp.original_byte;
                                if let Err(e) = self.backend.write_memory(&session.handle, address, &[original]).await {
                                    let _ = self.event_tx.send(EngineEvent::Error {
                                        message: format!("Failed to restore breakpoint byte: {e}"),
                                    });
                                }
                            }

                            // 2. Decrement RIP so it points to the original instruction
                            let mut ctx = match self.backend.get_registers(&session.handle, thread_id).await {
                                Ok(c) => c,
                                Err(e) => {
                                    let _ = self.event_tx.send(EngineEvent::Error {
                                        message: format!("Failed to get registers: {e}"),
                                    });
                                    let _ = self.event_tx.send(EngineEvent::BreakpointHit {
                                        kind: BreakpointKind::UserDefined(id),
                                        address,
                                        thread_id,
                                    });
                                    let to_state = self.current_state();
                                    self.log_transition(&from_state, &to_state);
                                    return Ok(());
                                }
                            };
                            ctx.rip = address;

                            // 3. Set Trap Flag for single step
                            ctx.rflags |= 0x100;
                            if let Err(e) = self.backend.set_registers(&session.handle, thread_id, &ctx).await {
                                let _ = self.event_tx.send(EngineEvent::Error {
                                    message: format!("Failed to set registers: {e}"),
                                });
                            }

                            // 4. Remember we are stepping over this breakpoint
                            self.stepping_over_breakpoint = Some(address);
                        }

                        // 5. Forward event to REPL
                        let _ = self.event_tx.send(EngineEvent::BreakpointHit {
                            kind: BreakpointKind::UserDefined(id),
                            address,
                            thread_id,
                        });

                        // 6. Tell debug loop to continue (will single-step over restored instruction)
                        if let Some(tx) = &self.debug_loop_tx {
                            let _ = tx.send(DebugLoopCommand::Continue);
                        }
                    }
                    BreakpointKind::Unknown => unreachable!("Unknown should have been resolved above"),
                }
            }
            EngineEvent::SingleStep { address, thread_id } => {
                info!("Single step at 0x{address:x} on thread {thread_id}");

                if let Some(session) = &self.session {
                    // If we were stepping over a breakpoint, reinstall it
                    if let Some(bp_addr) = self.stepping_over_breakpoint.take() {
                        if let Err(e) = self.backend.write_memory(&session.handle, bp_addr, &[0xCC]).await {
                            let _ = self.event_tx.send(EngineEvent::Error {
                                message: format!("Failed to reinstall breakpoint: {e}"),
                            });
                        }
                    }

                    // Clear trap flag
                    let mut ctx = match self.backend.get_registers(&session.handle, thread_id).await {
                        Ok(c) => c,
                        Err(_) => {
                            let _ = self.event_tx.send(EngineEvent::SingleStep { address, thread_id });
                            if let Some(tx) = &self.debug_loop_tx {
                                let _ = tx.send(DebugLoopCommand::Continue);
                            }
                            return Ok(());
                        }
                    };
                    ctx.rflags &= !0x100;
                    let _ = self.backend.set_registers(&session.handle, thread_id, &ctx).await;

                    // Continue automatically after single step
                    if let Some(tx) = &self.debug_loop_tx {
                        let _ = tx.send(DebugLoopCommand::Continue);
                    }
                }

                let _ = self.event_tx.send(EngineEvent::SingleStep { address, thread_id });
            }
            EngineEvent::Exception { code, address } => {
                info!("Exception 0x{code:08X} at 0x{address:x}");
                let _ = self.event_tx.send(EngineEvent::Exception { code, address });
                // For unknown exceptions, tell debug loop to report as not handled
                if let Some(tx) = &self.debug_loop_tx {
                    info!("Engine sending ContinueException to debug loop");
                    let _ = tx.send(DebugLoopCommand::ContinueException);
                }
            }
            EngineEvent::ProcessExited { code } => {
                info!("Process exited with code {code}");
                self.session = None;
                self.debug_loop_tx = None;
                let _ = self.event_tx.send(EngineEvent::ProcessExited { code });
            }
            EngineEvent::ProcessLaunched { pid } => {
                info!("Process launched: PID {pid}");
                let _ = self.event_tx.send(EngineEvent::ProcessLaunched { pid });
                // Auto-continue after initial process creation
                if let Some(tx) = &self.debug_loop_tx {
                    info!("Engine sending Continue to debug loop for process creation");
                    let _ = tx.send(DebugLoopCommand::Continue);
                }
            }
            EngineEvent::ProcessAttached { pid } => {
                info!("Process attached: PID {pid}");
                let _ = self.event_tx.send(EngineEvent::ProcessAttached { pid });
                if let Some(tx) = &self.debug_loop_tx {
                    let _ = tx.send(DebugLoopCommand::Continue);
                }
            }
            EngineEvent::ThreadCreated { id, handle } => {
                info!(target: "rde::engine::event", thread_id = id, handle, "ThreadCreated");
                if let Some(session) = &mut self.session {
                    let thread = Thread {
                        id,
                        handle: RawHandle(handle),
                        state: ThreadState::Running,
                        context: None,
                    };
                    session.threads.insert(id, thread);
                    if session.selected_thread.is_none() {
                        session.selected_thread = Some(id);
                    }
                }
                let _ = self.event_tx.send(EngineEvent::ThreadCreated { id, handle });
            }
            EngineEvent::ThreadExited { id, code } => {
                info!(target: "rde::engine::event", thread_id = id, exit_code = code, "ThreadExited");
                if let Some(session) = &mut self.session {
                    if let Some(thread) = session.threads.get_mut(&id) {
                        thread.state = ThreadState::Exited { exit_code: code };
                    }
                    if session.selected_thread == Some(id) {
                        // Fallback to oldest remaining non-exited thread
                        session.selected_thread = session
                            .threads
                            .values()
                            .filter(|t| t.state != ThreadState::Exited { exit_code: code })
                            .map(|t| t.id)
                            .min();
                    }
                }
                let _ = self.event_tx.send(EngineEvent::ThreadExited { id, code });
            }
            EngineEvent::ModuleLoaded { name, base } => {
                info!(target: "rde::engine::event", module_name = %name, base, "ModuleLoaded");
                if let Some(session) = &mut self.session {
                    // TODO: Integrate with symbol engine (rde-symbols) to call SymLoadModuleEx.
                    // This requires passing a SymbolEngine trait object into the engine,
                    // which needs architectural refactoring to avoid circular deps.
                    let module = Module {
                        name: name.clone(),
                        base_address: base,
                        size: 0,
                        path: None,
                        symbols_loaded: true, // Optimistically assume success for MVP
                    };
                    session.modules.insert(base, module);
                }
                let _ = self.event_tx.send(EngineEvent::ModuleLoaded { name, base });
            }
            EngineEvent::ModuleUnloaded { base } => {
                info!(target: "rde::engine::event", base, "ModuleUnloaded");
                if let Some(session) = &mut self.session {
                    // TODO: Call symbol_engine.unload_module(base) when integrated.
                    session.modules.remove(&base);
                }
                let _ = self.event_tx.send(EngineEvent::ModuleUnloaded { base });
            }
            _ => {
                let _ = self.event_tx.send(evt);
            }
        }
        Ok(())
    }



    #[instrument(skip(self))]
    async fn handle_disassemble(
        &mut self,
        address: Option<u64>,
        symbol: Option<String>,
        thread_id: Option<u32>,
        count: Option<usize>,
    ) -> Result<(), DebugError> {
        let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
        let count = count.unwrap_or(self.disasm_config.count).max(1).min(100);

        let (addr, rip) = match (address, symbol) {
            (Some(a), _) => (a, None),
            (None, Some(sym)) => {
                return Err(DebugError::Internal(format!(
                    "Symbol '{sym}' not found (symbol resolution not yet integrated)"
                )));
            }
            (None, None) => {
                let tid = thread_id
                    .or(session.selected_thread)
                    .or_else(|| session.threads.keys().copied().next())
                    .ok_or(DebugError::Internal("No threads available".into()))?;
                let ctx = self.backend.get_registers(&session.handle, tid).await?;
                let rip = ctx.rip;
                let back_count = count / 2;
                let back_bytes = (back_count * 8) as u64;
                let start = rip.saturating_sub(back_bytes);
                (start, Some(rip))
            }
        };

        let buffer_size = (64_usize.saturating_mul(count)).min(4096);
        let bytes = self.backend.read_memory(&session.handle, addr, buffer_size).await?;

        let active_bps = self.breakpoints.list_active();
        let bp_bytes = self.breakpoints.active_original_bytes();

        let mut disasm = Disassembler::new()?;
        let view = disasm.disassemble_at(addr, count, rip, &bytes, &active_bps, &bp_bytes);

        match view {
            Ok(view) => {
                let lines = view.lines.clone();
                let _ = self.event_tx.send(EngineEvent::Disassembly { lines });
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: view.format(),
                });
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
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

        // Write INT3 (0xCC)
        self.backend.write_memory(&session.handle, address, &[0xCC]).await?;

        let id = self.breakpoints.set(address, original[0]);
        let list: Vec<_> = self.breakpoints.list().into_iter().cloned().collect();
        let _ = self.event_tx.send(EngineEvent::BreakpointList { list });

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
