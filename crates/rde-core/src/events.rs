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
}

/// Events sent from the debug engine to the REPL/UI.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    ProcessLaunched {
        pid: u32,
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
    },
    ThreadExited {
        id: ThreadId,
        code: u32,
    },
    ModuleLoaded {
        name: String,
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
}
