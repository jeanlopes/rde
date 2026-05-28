//! RDE Win32 Backend — Direct Windows Debug API integration.
//!
//! # Safety
//! This crate contains `unsafe` blocks wrapping Win32 APIs. Every `unsafe` usage
//! MUST include a safety proof comment per the RDE Constitution (Principle VI).

use rde_core::{DebugBackend, DebugChannels, DebugError, ProcessHandle, RegisterContext, ThreadId};
use std::path::Path;

pub mod debug_loop;
pub mod memory;
pub mod module;
pub mod process;
pub mod thread;

/// Windows-specific backend implementation.
#[derive(Debug)]
pub struct WindowsBackend;

impl WindowsBackend {
    pub fn new() -> Self {
        Self
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
}
