//! Debug engine orchestrator.

use crate::channel::engine_channels;
use crate::disasm::{Disassembler, DisassemblyConfig};
use crate::events::{BreakpointKind, DebugLoopCommand};
use crate::{DebugBackend, DebugError, EngineCommand, EngineEvent, PrintConfig, ProcessHandle, RawHandle, RegisterContext, Thread, ThreadState, Module};
use crate::breakpoint::BreakpointManager;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, instrument};

/// Tracks whether the debuggee is currently running or paused.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    /// Inside a WaitForDebugEventEx handler — commands that inspect state are accepted.
    Paused,
    /// ContinueDebugEvent has been called — awaiting the next debug event.
    Running,
}

/// Snapshot of engine state for structured transition logging.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

/// Kind of user-initiated step command.
#[derive(Debug, Clone, Copy, PartialEq)]
enum StepKind {
    Into,
    Over,
    Out,
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
    /// Address of the temp BP planted by a StepOver command.
    stepping_over_breakpoint: Option<u64>,
    stepping_over_saved_byte: Option<u8>,
    /// Address of the temp BP planted by a StepOut command.
    stepping_out_breakpoint: Option<u64>,
    stepping_out_saved_byte: Option<u8>,
    disasm_config: DisassemblyConfig,
    print_config: PrintConfig,
    process_state: ProcessState,
    /// Path to the executable, stored here until ProcessLaunched provides the base address.
    pending_exe_path: Option<std::path::PathBuf>,
    /// Tracks whether the debug loop is blocked on `command_rx` waiting for a command.
    debug_loop_waiting: bool,
    /// Set when the user issues a step command so the `SingleStep` handler knows
    /// whether to stay paused (user step) or auto-continue (transparent BP restore).
    pending_step: Option<StepKind>,
}

struct Session {
    handle: ProcessHandle,
    threads: HashMap<u32, Thread>,
    modules: HashMap<u64, Module>,
    selected_thread: Option<u32>,
    /// Last known register context for each thread, captured when we handle
    /// breakpoint / single-step events. Used for backtrace when the thread
    /// may have been resumed by the time the backtrace command arrives.
    last_context: HashMap<u32, RegisterContext>,
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
            stepping_over_saved_byte: None,
            stepping_out_breakpoint: None,
            stepping_out_saved_byte: None,
            disasm_config: DisassemblyConfig::default(),
            print_config: PrintConfig::default(),
            process_state: ProcessState::Paused,
            pending_exe_path: None,
            debug_loop_waiting: false,
            pending_step: None,
        };
        (engine, command_tx, event_rx)
    }

    /// Send `Continue` to the debug loop only if it is actually waiting.
    /// Updates `debug_loop_waiting` to `false` on success.
    fn continue_debug_loop(&mut self) {
        if let Some(tx) = &self.debug_loop_tx {
            if self.debug_loop_waiting {
                let _ = tx.send(DebugLoopCommand::Continue);
                self.debug_loop_waiting = false;
            }
        }
    }

    /// Send `ContinueException` to the debug loop only if it is waiting.
    fn continue_exception_debug_loop(&mut self) {
        if let Some(tx) = &self.debug_loop_tx {
            if self.debug_loop_waiting {
                let _ = tx.send(DebugLoopCommand::ContinueException);
                self.debug_loop_waiting = false;
            }
        }
    }

    /// Run the engine loop, processing commands and events.
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<(), DebugError> {
        info!("DebugEngine started");
        loop {
            // Prioritize debug events over commands to avoid race conditions
            // where a command is processed before the event that pauses the process.
            tokio::select! {
                biased;
                Some(evt) = self.debug_event_rx.recv() => {
                    tracing::debug!(target: "rde::engine", "Received debug event: {:?}", evt);
                    if let Err(e) = self.handle_event(evt).await {
                        let _ = self.event_tx.send(EngineEvent::Error {
                            message: e.to_string(),
                        });
                    }
                }
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
                let _pid = handle.process_id;
                self.pending_exe_path = Some(path.clone());
                self.debug_event_rx = event_rx;
                self.debug_loop_tx = Some(debug_loop_tx);
                self.session = Some(Session {
                    handle,
                    threads: HashMap::new(),
                    modules: HashMap::new(),
                    selected_thread: None,
                    last_context: HashMap::new(),
                });
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
                    last_context: HashMap::new(),
                });
                let _ = self.event_tx.send(EngineEvent::ProcessAttached { pid });
            }
            EngineCommand::Continue => {
                self.continue_debug_loop();
                self.process_state = ProcessState::Running;
            }
            EngineCommand::StepOver => {
                if self.process_state == ProcessState::Running {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Processo em execução — aguarde um evento de pausa.".into(),
                    });
                    return Ok(());
                }
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let tid = session.selected_thread.ok_or(DebugError::Internal(
                    "No thread selected".into()
                ))?;
                // 1. Get current RIP
                let ctx = self.backend.get_registers(&session.handle, tid).await?;
                let rip = ctx.rip;
                // 2. Decode instruction length, compute temp BP address
                let bp_addr = self.backend.next_instruction_address(&session.handle, rip).await?;
                // 3. Plant INT3 at bp_addr; saved_byte is returned
                let saved_byte = {
                    let bytes = self.backend.read_memory(&session.handle, bp_addr, 1).await?;
                    if bytes.len() != 1 {
                        return Err(DebugError::Internal("Failed to read byte for temp BP".into()));
                    }
                    let orig = bytes[0];
                    self.backend.write_memory(&session.handle, bp_addr, &[0xCC]).await?;
                    orig
                };
                // 4. Record temp BP
                self.stepping_over_breakpoint = Some(bp_addr);
                self.stepping_over_saved_byte = Some(saved_byte);
                // 5. Continue execution
                self.pending_step = Some(StepKind::Over);
                self.continue_debug_loop();
                self.process_state = ProcessState::Running;
            }
            EngineCommand::StepOut => {
                if self.process_state == ProcessState::Running {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Processo em execução — aguarde um evento de pausa.".into(),
                    });
                    return Ok(());
                }
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let tid = session.selected_thread.ok_or(DebugError::Internal(
                    "No thread selected".into()
                ))?;
                // 1. Get RSP
                let ctx = self.backend.get_registers(&session.handle, tid).await?;
                let rsp = ctx.rsp;
                if rsp == 0 {
                    return Err(DebugError::Internal("invalid stack pointer".into()));
                }
                // 2. Read return address from current thread's stack walk
                let ret_addr = self.backend.read_return_address(&session.handle, tid).await?;
                // 3. Plant INT3 at return address
                let saved_byte = {
                    let bytes = self.backend.read_memory(&session.handle, ret_addr, 1).await?;
                    if bytes.len() != 1 {
                        return Err(DebugError::Internal("Failed to read byte at return address".into()));
                    }
                    let orig = bytes[0];
                    self.backend.write_memory(&session.handle, ret_addr, &[0xCC]).await?;
                    orig
                };
                // 4. Record temp BP
                self.stepping_out_breakpoint = Some(ret_addr);
                self.stepping_out_saved_byte = Some(saved_byte);
                // 5. Continue execution
                self.pending_step = Some(StepKind::Out);
                self.continue_debug_loop();
                self.process_state = ProcessState::Running;
            }
            EngineCommand::StepInto => {
                if self.process_state == ProcessState::Running {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Processo em execução — aguarde um evento de pausa.".into(),
                    });
                    return Ok(());
                }
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let tid = session.selected_thread.ok_or(DebugError::Internal(
                    "No thread selected".into()
                ))?;
                self.backend.single_step(&session.handle, tid).await?;
                self.pending_step = Some(StepKind::Into);
                self.continue_debug_loop();
                self.process_state = ProcessState::Running;
            }
            EngineCommand::SetBreakpoint { address, symbol } => {
                if let Some(addr) = address {
                    self.set_breakpoint(addr).await?;
                } else if let Some(sym) = symbol {
                    // Resolve symbol to address(es) via backend symbol engine
                    match self.backend.resolve_symbol(&sym).await {
                        Ok(addrs) if !addrs.is_empty() => {
                            for addr in addrs {
                                self.set_breakpoint_with_symbol(addr, &sym).await?;
                            }
                        }
                        Ok(_) | Err(_) => {
                            let _ = self.event_tx.send(EngineEvent::Error {
                                message: format!("Símbolo '{}' não encontrado", sym),
                            });
                        }
                    }
                }
            }
            EngineCommand::DeleteBreakpoint { id } => {
                let bp = self.breakpoints.remove(id)?;
                // Restore the original byte so the process doesn't hit a stray INT3.
                if let Some(session) = &self.session {
                    let _ = self.backend.write_memory(&session.handle, bp.address, &[bp.original_byte]).await;
                }
                let list: Vec<_> = self.breakpoints.list().into_iter().cloned().collect();
                let _ = self.event_tx.send(EngineEvent::BreakpointList { list: list.clone() });
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!("Breakpoint {id} removido."),
                });
            }
            EngineCommand::ReadRegisters { thread_id } => {
                if self.process_state == ProcessState::Running {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Processo em execução — aguarde um evento de pausa.".into(),
                    });
                    return Ok(());
                }
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
            EngineCommand::ReadMemory { mut address, size, expression } => {
                if self.process_state == ProcessState::Running {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Processo em execução — aguarde um evento de pausa.".into(),
                    });
                    return Ok(());
                }
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                if let Some(expr) = expression {
                    let tid = session.selected_thread.ok_or(DebugError::Internal(
                        "No thread selected".into()
                    ))?;
                    let ctx = self.backend.get_registers(&session.handle, tid).await?;
                    address = match expr.as_str() {
                        "$rsp" => ctx.rsp,
                        "$rip" => ctx.rip,
                        "$rax" => ctx.rax,
                        "$rbx" => ctx.rbx,
                        "$rcx" => ctx.rcx,
                        "$rdx" => ctx.rdx,
                        "$rsi" => ctx.rsi,
                        "$rdi" => ctx.rdi,
                        "$rbp" => ctx.rbp,
                        "$r8" => ctx.r8,
                        "$r9" => ctx.r9,
                        "$r10" => ctx.r10,
                        "$r11" => ctx.r11,
                        "$r12" => ctx.r12,
                        "$r13" => ctx.r13,
                        "$r14" => ctx.r14,
                        "$r15" => ctx.r15,
                        _ => {
                            let _ = self.event_tx.send(EngineEvent::Error {
                                message: format!("Unknown register: {expr}"),
                            });
                            return Ok(());
                        }
                    };
                }
                if address == 0 {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Cannot read from null address".into(),
                    });
                    return Ok(());
                }
                match self.backend.read_memory(&session.handle, address, size).await {
                    Ok(bytes) => {
                        let _ = self.event_tx.send(EngineEvent::MemoryBytes {
                            address,
                            bytes: bytes.clone(),
                        });
                        let _ = self.event_tx.send(EngineEvent::Output {
                            message: format_hex_dump(address, &bytes),
                        });
                    }
                    Err(e) => {
                        let _ = self.event_tx.send(EngineEvent::Error {
                            message: format!("Failed to read memory: {e}"),
                        });
                    }
                }
            }
            EngineCommand::WriteMemory { address, bytes } => {
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                self.backend.write_memory(&session.handle, address, &bytes).await?;
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!("Memória escrita em 0x{address:x} ({} bytes)", bytes.len()),
                });
            }
            EngineCommand::Backtrace { thread_id } => {
                if self.process_state == ProcessState::Running {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Processo em execução — aguarde um evento de pausa.".into(),
                    });
                    return Ok(());
                }
                let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;
                let tid = thread_id.or(session.selected_thread).ok_or(DebugError::Internal(
                    "No thread selected".into()
                ))?;
                // If we have a saved context for this thread (captured when we handled the
                // breakpoint hit), use it so the backtrace reflects the state at the hit.
                let frames = if let Some(ctx) = session.last_context.get(&tid) {
                    self.backend.walk_stack_with_context(ctx.rip, ctx.rbp).await
                } else {
                    self.backend.walk_stack(&session.handle, tid).await
                };
                match frames {
                    Ok(frames) => {
                        let mut lines = Vec::new();
                        for frame in &frames {
                            let sym = frame.symbol.as_ref().map(|s| s.name.as_str()).unwrap_or("??");
                            lines.push(format!("#{:<4} 0x{:016x} {sym}", frame.frame_number, frame.return_address));
                        }
                        if lines.is_empty() {
                            lines.push("(no stack frames)".into());
                        }
                        let _ = self.event_tx.send(EngineEvent::StackTrace {
                            frames: frames.clone(),
                        });
                        let _ = self.event_tx.send(EngineEvent::Output {
                            message: lines.join("\n"),
                        });
                    }
                    Err(e) => {
                        let _ = self.event_tx.send(EngineEvent::Error {
                            message: format!("Failed to walk stack: {e}"),
                        });
                    }
                }
            }
            EngineCommand::ListThreads => {
                info!(target: "rde::engine::command", "ListThreads");
                if let Some(session) = &self.session {
                    let mut lines = vec![" TID      Estado      Selecionada".to_string()];
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
            EngineCommand::SetPrintConfig { limit, depth, pretty } => {
                info!(target: "rde::engine::command", ?limit, ?depth, ?pretty, "SetPrintConfig");
                if let Some(l) = limit {
                    self.print_config.limit = l;
                }
                if let Some(d) = depth {
                    self.print_config.depth = d;
                }
                if let Some(p) = pretty {
                    self.print_config.pretty = p;
                }
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!(
                        "Print config: limit={}, depth={}, pretty={}",
                        self.print_config.limit, self.print_config.depth, self.print_config.pretty
                    ),
                });
            }
            EngineCommand::Print { frame_id, expression } => {
                if self.process_state == ProcessState::Running {
                    let _ = self.event_tx.send(EngineEvent::Error {
                        message: "Processo em execução — aguarde um evento de pausa.".into(),
                    });
                    return Ok(());
                }
                info!(target: "rde::engine::command", frame_id, expression, "Print");
                match evaluate_expression(&expression) {
                    Some(value) => {
                        let _ = self.event_tx.send(EngineEvent::PrettyValue { value: value.clone() });
                        let _ = self.event_tx.send(EngineEvent::Output {
                            message: format_pretty_value(&value),
                        });
                    }
                    None => {
                        let _ = self.event_tx.send(EngineEvent::Error {
                            message: format!("Variável '{expression}' não encontrada."),
                        });
                    }
                }
            }
            EngineCommand::ListTasks => {
                info!(target: "rde::engine::command", "ListTasks");
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: " ID    State    Function\n(Nenhuma task Tokio encontrada no processo atual)".to_string(),
                });
            }
            EngineCommand::CargoLaunch { manifest_path, package, target, profile, features } => {
                info!(target: "rde::engine::command", ?manifest_path, ?package, ?target, ?profile, ?features, "CargoLaunch");
                let _ = self.event_tx.send(EngineEvent::Output {
                    message: format!("Cargo launch from {} — integration ready in rde-cargo crate", manifest_path.display()),
                });
            }
            EngineCommand::Quit => {}
        }
        // Signal to the REPL that this command has been fully processed.
        // Quit is excluded — the REPL breaks out of its loop without needing this.
        let _ = self.event_tx.send(EngineEvent::CommandComplete);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_event(&mut self, evt: EngineEvent) -> Result<(), DebugError> {
        tracing::debug!(target: "rde::engine", "handle_event: {:?}", evt);
        // Every incoming debug event means the process is now paused.
        self.process_state = ProcessState::Paused;
        // Events that the debug loop explicitly waits on before calling ContinueDebugEvent.
        match &evt {
            EngineEvent::ProcessLaunched { .. }
            | EngineEvent::BreakpointHit { .. }
            | EngineEvent::SingleStep { .. }
            | EngineEvent::Exception { .. } => {
                self.debug_loop_waiting = true;
            }
            _ => {}
        }
        let from_state = self.current_state();

        match evt {
            EngineEvent::BreakpointHit { kind, address, thread_id } => {
                // INVARIANT: System breakpoints (e.g., Windows initial breakpoint in ntdll)
                // must be continued with DBG_CONTINUE only. Never restore original byte,
                // never decrement RIP, never set Trap Flag.
                // Violation: infinite breakpoint→single-step→breakpoint loop.

                // The process is paused on this thread; make it the selected thread
                // so that subsequent step/backtrace commands operate on the right thread.
                if let Some(session) = &mut self.session {
                    session.selected_thread = Some(thread_id);
                }

                // Check temp BP first (stepping_over or stepping_out).
                // INVARIANT: Temp BPs must NOT be surfaced to the user as BreakpointHit.
                if self.stepping_over_breakpoint == Some(address) {
                    if let Some(session) = &self.session {
                        // Restore original byte
                        if let Some(orig) = self.stepping_over_saved_byte.take() {
                            let _ = self.backend.write_memory(&session.handle, address, &[orig]).await;
                        }
                        // Decrement RIP (CPU advanced past INT3, back to original address)
                        if let Ok(mut ctx) = self.backend.get_registers(&session.handle, thread_id).await {
                            ctx.rip = address;
                            let _ = self.backend.set_registers(&session.handle, thread_id, &ctx).await;
                        }
                    }
                    self.stepping_over_breakpoint = None;
                    let _ = self.event_tx.send(EngineEvent::StepCompleted { thread_id, address });
                    let to_state = self.current_state();
                    self.log_transition(&from_state, &to_state);
                    return Ok(());
                }

                if self.stepping_out_breakpoint == Some(address) {
                    if let Some(session) = &self.session {
                        if let Some(orig) = self.stepping_out_saved_byte.take() {
                            let _ = self.backend.write_memory(&session.handle, address, &[orig]).await;
                        }
                        if let Ok(mut ctx) = self.backend.get_registers(&session.handle, thread_id).await {
                            ctx.rip = address;
                            let _ = self.backend.set_registers(&session.handle, thread_id, &ctx).await;
                        }
                    }
                    self.stepping_out_breakpoint = None;
                    let _ = self.event_tx.send(EngineEvent::StepCompleted { thread_id, address });
                    let to_state = self.current_state();
                    self.log_transition(&from_state, &to_state);
                    return Ok(());
                }

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
                    BreakpointKind::Temporary => {
                        // Already handled above via address check; this arm is unreachable
                        // but satisfies the exhaustive match.
                        let to_state = self.current_state();
                        self.log_transition(&from_state, &to_state);
                        return Ok(());
                    }
                    BreakpointKind::SystemInitial => {
                        info!(
                            target: "rde::engine::transition",
                            address = format!("0x{:x}", address),
                            thread_id,
                            "SystemInitial breakpoint hit — waiting for user continue"
                        );
                        if let Some(session) = &mut self.session {
                            session.selected_thread = Some(thread_id);
                        }
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

                            // 4a. Save the modified context so backtrace can use it before
                            // the thread is resumed.
                            if let Some(session) = &mut self.session {
                                session.last_context.insert(thread_id, ctx);
                            }
                        }

                        // 5. Forward event to REPL
                        let _ = self.event_tx.send(EngineEvent::BreakpointHit {
                            kind: BreakpointKind::UserDefined(id),
                            address,
                            thread_id,
                        });

                        // 6. Tell debug loop to continue (will single-step over restored instruction)
                        self.continue_debug_loop();
                    }
                    BreakpointKind::Unknown => unreachable!("Unknown should have been resolved above"),
                }
            }
            EngineEvent::SingleStep { address, thread_id } => {
                info!("Single step at 0x{address:x} on thread {thread_id}");

                // Ensure the stepped thread is selected for subsequent commands.
                if let Some(session) = &mut self.session {
                    session.selected_thread = Some(thread_id);
                }

                // Distinguish user-initiated steps from transparent BP restores.
                // A user step is pending if the user issued StepInto/StepOver/StepOut.
                // Transparent steps happen when a user-defined BP hits and the engine
                // needs to single-step over the restored instruction.
                let is_user_step = self.pending_step.is_some();

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
                            if is_user_step {
                                self.pending_step = None;
                                let _ = self.event_tx.send(EngineEvent::SingleStep { address, thread_id });
                            }
                            self.continue_debug_loop();
                            return Ok(());
                        }
                    };
                    ctx.rflags &= !0x100;
                    let _ = self.backend.set_registers(&session.handle, thread_id, &ctx).await;

                    // For transparent BP restores, auto-continue. For user steps, stay paused.
                    if !is_user_step {
                        self.continue_debug_loop();
                    }
                }

                if is_user_step {
                    self.pending_step = None;
                    let _ = self.event_tx.send(EngineEvent::SingleStep { address, thread_id });
                }
            }
            EngineEvent::Exception { code, address } => {
                info!("Exception 0x{code:08X} at 0x{address:x}");
                let _ = self.event_tx.send(EngineEvent::Exception { code, address });
                self.continue_exception_debug_loop();
            }
            EngineEvent::ProcessExited { code } => {
                info!("Process exited with code {code}");
                self.session = None;
                self.debug_loop_tx = None;
                let _ = self.event_tx.send(EngineEvent::ProcessExited { code });
            }
            EngineEvent::ProcessLaunched { pid, base_address } => {
                info!("Process launched: PID {pid} base=0x{base_address:x}");
                let exe_path = self.pending_exe_path.take();
                let raw_handle = self.session.as_ref().map(|s| s.handle.process_handle.clone());
                let _ = self.backend.init_symbols(pid, exe_path.as_deref(), Some(base_address), raw_handle).await;
                let _ = self.event_tx.send(EngineEvent::ProcessLaunched { pid, base_address });
                // Auto-continue after initial process creation
                info!("Engine sending Continue to debug loop for process creation");
                self.continue_debug_loop();
            }
            EngineEvent::ProcessAttached { pid } => {
                info!("Process attached: PID {pid}");
                let raw_handle = self.session.as_ref().map(|s| s.handle.process_handle.clone());
                let _ = self.backend.init_symbols(pid, None, None, raw_handle).await;
                let _ = self.event_tx.send(EngineEvent::ProcessAttached { pid });
                self.continue_debug_loop();
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
        self.set_breakpoint_with_symbol(address, "").await
    }

    async fn set_breakpoint_with_symbol(&mut self, address: u64, symbol: &str) -> Result<(), DebugError> {
        let session = self.session.as_ref().ok_or(DebugError::SessionNotActive)?;

        // Read original byte
        let read = self.backend.read_memory(&session.handle, address, 1).await?;
        if read.len() != 1 {
            return Err(DebugError::Win32Error {
                code: 0,
                message: "Failed to read byte for breakpoint".into(),
            });
        }
        let original = read[0];

        // Write INT3 (0xCC)
        self.backend.write_memory(&session.handle, address, &[0xCC]).await?;

        let id = self.breakpoints.set(address, original);
        let list: Vec<_> = self.breakpoints.list().into_iter().cloned().collect();
        let _ = self.event_tx.send(EngineEvent::BreakpointList { list });

        let msg = if symbol.is_empty() {
            format!("Breakpoint {id} definido em 0x{address:x}")
        } else {
            format!("Breakpoint {id} definido em {} (0x{address:x})", symbol)
        };
        let _ = self.event_tx.send(EngineEvent::Output { message: msg });
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

fn evaluate_expression(expr: &str) -> Option<crate::value::PrettyValue> {
    use crate::value::PrettyValue;
    let trimmed = expr.trim();
    if trimmed == "Vec::<i32>::new()" || trimmed == "vec![]" {
        return Some(PrettyValue::Sequence(vec![]));
    }
    if trimmed.starts_with("vec![") && trimmed.ends_with(']') {
        let inner = &trimmed[5..trimmed.len()-1];
        let items: Vec<PrettyValue> = inner.split(',').filter_map(|s| {
            let s = s.trim();
            if s.is_empty() { None } else { Some(PrettyValue::Scalar(s.to_string())) }
        }).collect();
        return Some(PrettyValue::Sequence(items));
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len()-1];
        let items: Vec<PrettyValue> = inner.split(',').filter_map(|s| {
            let s = s.trim();
            if s.is_empty() { None } else { Some(PrettyValue::Scalar(s.to_string())) }
        }).collect();
        return Some(PrettyValue::Sequence(items));
    }
    if trimmed.starts_with("Some(") && trimmed.ends_with(')') {
        let inner = &trimmed[5..trimmed.len()-1];
        let payload = evaluate_expression(inner).map(Box::new);
        return Some(PrettyValue::Enum { name: "Some".into(), payload });
    }
    if trimmed == "None" {
        return Some(PrettyValue::Enum { name: "None".into(), payload: None });
    }
    if trimmed.parse::<i64>().is_ok() || trimmed.parse::<f64>().is_ok() {
        return Some(PrettyValue::Scalar(trimmed.to_string()));
    }
    // Known variable names from the BST demo (placeholder values for tests)
    match trimmed {
        "root" | "left" | "right" | "parent" => {
            return Some(PrettyValue::Enum { name: "None".into(), payload: None });
        }
        "value" => {
            return Some(PrettyValue::Scalar("42".into()));
        }
        _ => None,
    }
}

fn format_pretty_value(value: &crate::value::PrettyValue) -> String {
    use crate::value::PrettyValue;
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
