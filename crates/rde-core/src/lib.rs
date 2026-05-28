//! RDE Core — Traits, types, and engine state machine for the Rust Debugger Engine.

use std::path::PathBuf;

pub mod breakpoint;
pub mod cargo;
pub mod channel;
pub mod disasm;
pub mod engine;
pub mod events;
pub mod memory;
pub mod task;
pub mod value;

pub use engine::DebugEngine;
pub use events::{BreakpointKind, DebugLoopCommand, EngineCommand, EngineEvent};
pub use disasm::{DisassemblyLine, DisassemblyView, DisassemblyConfig};
pub use breakpoint::{Breakpoint, BreakpointManager, BreakpointState};

/// Unique identifier for a debug session.
pub type SessionId = uuid::Uuid;

/// OS thread identifier.
pub type ThreadId = u32;

/// Unique identifier for a breakpoint.
pub type BreakpointId = u64;

/// Opaque handle to a debugged process.
#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pub process_id: u32,
    pub process_handle: RawHandle,
}

/// Opaque raw handle wrapper.
#[derive(Debug, Clone)]
pub struct RawHandle(pub usize);

/// Target to debug: either launch a new process or attach to an existing one.
#[derive(Debug, Clone)]
pub enum Target {
    Launch {
        path: PathBuf,
        args: Vec<String>,
        working_dir: Option<PathBuf>,
    },
    Attach {
        pid: u32,
    },
}

/// Current state of a debug session.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Running,
    Paused { reason: PauseReason },
    Exited { code: u32 },
}

/// Reason why the session entered the Paused state.
#[derive(Debug, Clone, PartialEq)]
pub enum PauseReason {
    BreakpointHit {
        address: u64,
        breakpoint_id: BreakpointId,
    },
    SingleStepComplete {
        address: u64,
    },
    Exception {
        code: u32,
        address: u64,
    },
    ProcessExited,
}

/// CPU register context for x86-64.
#[derive(Debug, Clone, Default)]
pub struct RegisterContext {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub rip: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rflags: u64,
}

/// Represents a thread in the target process.
#[derive(Debug, Clone)]
pub struct Thread {
    pub id: ThreadId,
    pub handle: RawHandle,
    pub state: ThreadState,
    pub context: Option<RegisterContext>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadState {
    Running,
    Suspended,
    Exited { exit_code: u32 },
}

/// Loaded module (DLL or EXE) in the target address space.
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub base_address: u64,
    pub size: u64,
    pub path: Option<PathBuf>,
    pub symbols_loaded: bool,
}

/// Stack frame with optional symbol info.
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub frame_number: u32,
    pub return_address: u64,
    pub frame_pointer: u64,
    pub stack_pointer: u64,
    pub symbol: Option<SymbolInfo>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub module: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub address: u64,
}

/// Core debugger errors.
#[derive(thiserror::Error, Debug)]
pub enum DebugError {
    #[error("Process not found: {pid}")]
    ProcessNotFound { pid: u32 },
    #[error("Access denied to process {pid}")]
    AccessDenied { pid: u32 },
    #[error("Invalid address: 0x{address:x}")]
    InvalidAddress { address: u64 },
    #[error("Breakpoint {id} not found")]
    BreakpointNotFound { id: BreakpointId },
    #[error("No active debug session")]
    SessionNotActive,
    #[error("Process is already running")]
    AlreadyRunning,
    #[error("Process is not paused")]
    NotPaused,
    #[error("Win32 error {code}: {message}")]
    Win32Error { code: u32, message: String },
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Channels produced by a successful launch/attach.
pub type DebugChannels = (
    tokio::sync::mpsc::UnboundedReceiver<EngineEvent>,
    tokio::sync::mpsc::UnboundedSender<DebugLoopCommand>,
);

/// Trait abstracting the debugger backend (Win32, mock, future backends).
#[async_trait::async_trait]
pub trait DebugBackend: Send + Sync {
    /// Launch a new process under the debugger.
    /// Returns the process handle and channels for communicating with the debug loop.
    /// The debug loop runs in a dedicated thread that also creates the process,
    /// because Windows requires `CreateProcessW` and `WaitForDebugEvent` to run
    /// on the same thread.
    async fn launch(&self, path: &std::path::Path, args: &[String]) -> Result<(ProcessHandle, DebugChannels), DebugError>;
    async fn attach(&self, pid: u32) -> Result<(ProcessHandle, DebugChannels), DebugError>;
    async fn continue_execution(&self, handle: &ProcessHandle) -> Result<(), DebugError>;
    async fn single_step(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError>;
    async fn read_memory(&self, handle: &ProcessHandle, address: u64, size: usize) -> Result<Vec<u8>, DebugError>;
    async fn write_memory(&self, handle: &ProcessHandle, address: u64, bytes: &[u8]) -> Result<(), DebugError>;
    async fn get_registers(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<RegisterContext, DebugError>;
    async fn set_registers(&self, handle: &ProcessHandle, thread_id: ThreadId, ctx: &RegisterContext) -> Result<(), DebugError>;
    async fn suspend_thread(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError>;
    async fn resume_thread(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError>;
}
