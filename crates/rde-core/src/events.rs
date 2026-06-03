//! Engine command and event types.

use crate::{BreakpointId, ThreadId};
use std::path::PathBuf;

/// Commands sent from the REPL/UI to the debug engine.
#[derive(Debug, Clone)]
pub enum EngineCommand {
    Launch {
        path: PathBuf,
        args: Vec<String>,
    },
    Attach {
        pid: u32,
    },
    Continue,
    StepInto,
    /// Advance one machine instruction without entering callees.
    StepOver,
    /// Run to the end of the current function and stop at the caller.
    StepOut,
    SetBreakpoint {
        address: Option<u64>,
        symbol: Option<String>,
    },
    DeleteBreakpoint {
        id: BreakpointId,
    },
    ReadRegisters {
        thread_id: Option<ThreadId>,
    },
    ReadMemory {
        address: u64,
        size: usize,
        expression: Option<String>,
    },
    WriteMemory {
        address: u64,
        bytes: Vec<u8>,
    },
    Backtrace {
        thread_id: Option<ThreadId>,
    },
    ListThreads,
    ListModules,
    Print {
        frame_id: u64,
        expression: String,
    },
    ListTasks,
    CargoLaunch {
        manifest_path: std::path::PathBuf,
        package: Option<String>,
        target: Option<String>,
        profile: String,
        features: Vec<String>,
    },
    SelectThread {
        id: ThreadId,
    },
    Disassemble {
        address: Option<u64>,
        symbol: Option<String>,
        thread_id: Option<ThreadId>,
        count: Option<usize>,
    },
    SetDisassemblyConfig {
        auto_show: Option<bool>,
        count: Option<usize>,
    },
    /// Configure pretty-printer output limits.
    SetPrintConfig {
        limit: Option<usize>,
        depth: Option<usize>,
        pretty: Option<bool>,
    },
    Quit,
}

/// Commands sent from the engine to the debug loop thread.
#[derive(Debug, Clone)]
pub enum DebugLoopCommand {
    Continue,
    ContinueException,
}

/// Classification of a breakpoint hit.
///
/// This enum separates system-generated breakpoints from user-defined ones,
/// preventing the engine from applying user-breakpoint protocol (restore byte,
/// decrement RIP, set Trap Flag) to system breakpoints like the Windows
/// initial breakpoint in ntdll.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakpointKind {
    /// The debug loop could not determine the breakpoint type.
    /// The engine must resolve this using the breakpoint manager.
    Unknown,
    /// A system-generated breakpoint (e.g., the Windows initial breakpoint
    /// in ntdll before process entry). Must be continued with DBG_CONTINUE
    /// only — never manipulate registers or memory.
    SystemInitial,
    /// A user-defined software breakpoint (INT3). The full protocol applies:
    /// restore original byte, decrement RIP, set Trap Flag.
    UserDefined(BreakpointId),
    /// An internal temporary breakpoint planted by step-over or step-out logic.
    /// MUST NOT be surfaced as a hit event to the user.
    Temporary,
}

/// Events sent from the debug engine to the REPL/UI.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    ProcessLaunched {
        pid: u32,
        /// Base address of the main executable (from CREATE_PROCESS_DEBUG_EVENT).
        base_address: u64,
    },
    ProcessAttached {
        pid: u32,
    },
    BreakpointHit {
        kind: BreakpointKind,
        address: u64,
        thread_id: ThreadId,
    },
    SingleStep {
        address: u64,
        thread_id: ThreadId,
    },
    ProcessExited {
        code: u32,
    },
    ThreadCreated {
        id: ThreadId,
        handle: usize,
    },
    ThreadExited {
        id: ThreadId,
        code: u32,
    },
    ModuleLoaded {
        name: String,
        base: u64,
    },
    ModuleUnloaded {
        base: u64,
    },
    Exception {
        code: u32,
        address: u64,
    },
    Error {
        message: String,
    },
    Output {
        message: String,
    },
    /// Structured register context response.
    Registers {
        ctx: crate::RegisterContext,
    },
    /// Structured disassembly response.
    Disassembly {
        lines: Vec<crate::DisassemblyLine>,
    },
    /// Structured breakpoint list.
    BreakpointList {
        list: Vec<crate::Breakpoint>,
    },
    /// Structured stack trace response.
    StackTrace {
        frames: Vec<crate::StackFrame>,
    },
    /// Pretty-printed value response.
    PrettyValue {
        value: crate::value::PrettyValue,
    },
    /// Tokio async task list response.
    TaskList {
        tasks: Vec<crate::task::AsyncTask>,
    },
    /// Emitted after a StepOver or StepOut operation completes.
    StepCompleted {
        thread_id: ThreadId,
        /// Address the IP is now at (after the step).
        address: u64,
    },
    /// Cargo launch result.
    CargoLaunchResult {
        result: Result<u32, String>,
    },
    /// Raw memory bytes response (for pretty-print integration).
    MemoryBytes {
        address: u64,
        bytes: Vec<u8>,
    },
    /// Emitted by the engine after every handle_command completes.
    /// The REPL waits for this before printing the next prompt.
    CommandComplete,
}
