//! RDE Win32 Backend — Direct Windows Debug API integration.
//!
//! # Safety
//! This crate contains `unsafe` blocks wrapping Win32 APIs. Every `unsafe` usage
//! MUST include a safety proof comment per the RDE Constitution (Principle VI).

use rde_core::{DebugBackend, DebugError, ProcessHandle, RegisterContext, ThreadId};
use rde_core::events::DebugLoopCommand;
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
    async fn launch(&self, path: &Path, args: &[String]) -> Result<ProcessHandle, DebugError> {
        process::launch(path, args).await
    }

    async fn attach(&self, pid: u32) -> Result<ProcessHandle, DebugError> {
        process::attach(pid).await
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

    fn on_session_started(
        &self,
        handle: &ProcessHandle,
        event_tx: tokio::sync::mpsc::UnboundedSender<rde_core::EngineEvent>,
        command_rx: tokio::sync::mpsc::UnboundedReceiver<DebugLoopCommand>,
    ) {
        let _ = debug_loop::start_debug_loop(handle, event_tx, command_rx);
    }
}
