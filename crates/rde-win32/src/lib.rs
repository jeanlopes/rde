//! RDE Win32 Backend — Direct Windows Debug API integration.
//!
//! # Safety
//! This crate contains `unsafe` blocks wrapping Win32 APIs. Every `unsafe` usage
//! MUST include a safety proof comment per the RDE Constitution (Principle VI).

use rde_core::{DebugBackend, DebugChannels, DebugError, ProcessHandle, RegisterContext, ThreadId};
use rde_symbols::{DbgHelpSymbolEngine, SymbolEngine};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub mod debug_loop;
pub mod memory;
pub mod module;
pub mod process;
pub mod step;
pub mod thread;

/// Windows-specific backend implementation.
pub struct WindowsBackend {
    symbol_engine_arc: Arc<Mutex<Option<DbgHelpSymbolEngine>>>,
}

impl std::fmt::Debug for WindowsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsBackend").finish()
    }
}

impl WindowsBackend {
    pub fn new() -> Self {
        Self {
            symbol_engine_arc: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl DebugBackend for WindowsBackend {
    async fn launch(&self, path: &Path, args: &[String]) -> Result<(ProcessHandle, DebugChannels), DebugError> {
        let path = path.to_path_buf();
        let args = args.to_vec();
        
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (debug_loop_tx, debug_loop_rx) = tokio::sync::mpsc::unbounded_channel();
        let (handle_tx, handle_rx) = tokio::sync::oneshot::channel();
        
        // Spawn debug loop thread.  CRITICAL: on Windows, CreateProcessW with
        // DEBUG_PROCESS and WaitForDebugEvent must run on the *same* thread.
        std::thread::spawn(move || {
            match process::launch(&path, &args) {
                Ok(handle) => {
                    // Send handle back so the engine can use it immediately.
                    let _ = handle_tx.send(handle.clone());
                    // Enter the debug event loop on this thread.
                    debug_loop::run_debug_loop(&handle, event_tx, debug_loop_rx);
                }
                Err(e) => {
                    tracing::error!("Failed to launch process in debug loop thread: {:?}", e);
                    // handle_tx is dropped here, so the receiver gets an error.
                }
            }
        });
        
        // Wait for the debug loop thread to create the process and send back the handle.
        let handle = handle_rx.await.map_err(|_| {
            DebugError::Internal("Debug loop thread died before creating process".into())
        })?;
        
        Ok((handle, (event_rx, debug_loop_tx)))
    }

    async fn attach(&self, pid: u32) -> Result<(ProcessHandle, DebugChannels), DebugError> {
        let handle = process::attach(pid)?;
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (debug_loop_tx, debug_loop_rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Signal attachment before the debug loop starts sending events.
        let _ = event_tx.send(rde_core::EngineEvent::ProcessAttached { pid });
        
        let handle_for_thread = handle.clone();
        std::thread::spawn(move || {
            debug_loop::run_debug_loop(&handle_for_thread, event_tx, debug_loop_rx);
        });
        
        Ok((handle, (event_rx, debug_loop_tx)))
    }

    async fn continue_execution(&self, _handle: &ProcessHandle) -> Result<(), DebugError> {
        // Continue is handled by the debug loop thread calling ContinueDebugEvent
        Ok(())
    }

    async fn single_step(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError> {
        let mut ctx = thread::get_context(handle, thread_id)?;
        ctx.rflags |= 0x100; // Set Trap flag (EF)
        thread::set_context(handle, thread_id, &ctx)
    }

    async fn read_memory(&self, handle: &ProcessHandle, address: u64, size: usize) -> Result<Vec<u8>, DebugError> {
        let mut buf = vec![0u8; size];
        let read = memory::read_memory(handle, address, size, &mut buf)?;
        buf.truncate(read);
        Ok(buf)
    }

    async fn write_memory(&self, handle: &ProcessHandle, address: u64, bytes: &[u8]) -> Result<(), DebugError> {
        memory::write_memory(handle, address, bytes)
    }

    async fn get_registers(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<RegisterContext, DebugError> {
        thread::get_context(handle, thread_id)
    }

    async fn set_registers(&self, handle: &ProcessHandle, thread_id: ThreadId, ctx: &RegisterContext) -> Result<(), DebugError> {
        thread::set_context(handle, thread_id, ctx)
    }

    async fn suspend_thread(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError> {
        thread::suspend(handle, thread_id)
    }

    async fn resume_thread(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError> {
        thread::resume(handle, thread_id)
    }

    async fn next_instruction_address(&self, handle: &ProcessHandle, rip: u64) -> Result<u64, DebugError> {
        step::next_instruction_address(handle, rip)
    }

    async fn read_return_address(&self, _handle: &ProcessHandle, thread_id: ThreadId) -> Result<u64, DebugError> {
        step::read_return_address(thread_id, &self.symbol_engine_arc)
    }

    async fn walk_stack(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<Vec<rde_core::StackFrame>, DebugError> {
        let engine = self.symbol_engine_arc.lock().map_err(|e| DebugError::Internal(format!("Symbol engine lock poisoned: {e}")))?;
        let engine = engine.as_ref().ok_or_else(|| DebugError::Internal("Symbol engine not initialized".into()))?;
        // Use thread::get_context to obtain RIP/RBP — this path is known to work
        // (the engine already calls it successfully on breakpoint hits).
        match thread::get_context(handle, thread_id) {
            Ok(regs) => engine.walk_stack_from_context(regs.rip, regs.rbp),
            Err(e) => {
                // Fallback to the engine's internal OpenThread/GetThreadContext path.
                engine.walk_stack(thread_id)
            }
        }
    }

    async fn walk_stack_with_context(&self, rip: u64, rbp: u64) -> Result<Vec<rde_core::StackFrame>, DebugError> {
        let engine = self.symbol_engine_arc.lock().map_err(|e| DebugError::Internal(format!("Symbol engine lock poisoned: {e}")))?;
        let engine = engine.as_ref().ok_or_else(|| DebugError::Internal("Symbol engine not initialized".into()))?;
        engine.walk_stack_from_context(rip, rbp)
    }

    async fn init_symbols(&self, process_id: u32, exe_path: Option<&std::path::Path>, base_address: Option<u64>, process_handle: Option<rde_core::RawHandle>) -> Result<(), DebugError> {
        // DbgHelp is blocking and must NOT block the tokio event loop.
        // Run symbol initialization on a blocking thread and await completion
        // so that subsequent commands (e.g. break main) see a ready engine.
        let sym = Arc::clone(&self.symbol_engine_arc);
        let local_dir = exe_path
            .and_then(|p| p.parent())
            .map(|d| {
                let s = d.to_string_lossy();
                if let Some(stripped) = s.strip_prefix(r"\\?\") {
                    stripped.to_string()
                } else {
                    s.into_owned()
                }
            });
        let exe_path_owned = exe_path.map(|p| p.to_path_buf());
        let raw_handle = process_handle.map(|h| h.0 as isize);
        let handle = tokio::task::spawn_blocking(move || {
            match DbgHelpSymbolEngine::new() {
                Ok(mut engine) => {
                    if let Some(dir) = local_dir {
                        engine.set_symbol_path(&dir);
                    }
                    if let Err(e) = engine.initialize(process_id, exe_path_owned.as_deref(), base_address, raw_handle) {
                        tracing::warn!("DbgHelp initialize failed: {:?}", e);
                        {
                            use std::fs::OpenOptions;
                            use std::io::Write;
                            let mut f = OpenOptions::new().create(true).append(true).open("C:/workspace/rde/sym_debug.log").unwrap();
                            writeln!(f, "init_symbols: initialize failed: {:?}", e).unwrap();
                        }
                        return;
                    }
                    let mut guard = sym.lock().unwrap();
                    *guard = Some(engine);
                    {
                        use std::fs::OpenOptions;
                        use std::io::Write;
                        let mut f = OpenOptions::new().create(true).append(true).open("C:/workspace/rde/sym_debug.log").unwrap();
                        writeln!(f, "init_symbols: guard set to Some").unwrap();
                    }
                    tracing::info!("DbgHelp symbols initialized for PID {process_id}");
                }
                Err(e) => {
                    tracing::warn!("DbgHelpSymbolEngine::new failed: {:?}", e);
                    {
                        use std::fs::OpenOptions;
                        use std::io::Write;
                        let mut f = OpenOptions::new().create(true).append(true).open("C:/workspace/rde/sym_debug.log").unwrap();
                        writeln!(f, "init_symbols: DbgHelpSymbolEngine::new failed: {:?}", e).unwrap();
                    }
                }
            }
        });
        // Await completion so symbol engine is guaranteed ready before we return
        let _ = handle.await;
        Ok(())
    }

    async fn resolve_symbol(&self, name: &str) -> Result<Vec<u64>, DebugError> {
        use std::sync::Arc;
        let sym = Arc::clone(&self.symbol_engine_arc);
        let name_owned = name.to_string();
        tokio::task::spawn_blocking(move || {
            {
                use std::fs::OpenOptions;
                use std::io::Write;
                let mut f = OpenOptions::new().create(true).append(true).open("C:/workspace/rde/sym_debug.log").unwrap();
                let guard = sym.lock().unwrap();
                match guard.as_ref() {
                    Some(_) => writeln!(f, "resolve_symbol: engine is Some").unwrap(),
                    None => writeln!(f, "resolve_symbol: engine is None").unwrap(),
                }
            }
            let guard = sym.lock().unwrap();
            match guard.as_ref() {
                Some(engine) => engine.resolve_by_name(&name_owned),
                None => Err(DebugError::Internal("Symbol engine not initialized".into())),
            }
        }).await.map_err(|e| DebugError::Internal(format!("spawn_blocking failed: {e}")))?
    }
}
