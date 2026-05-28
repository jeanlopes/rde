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
        id: BreakpointId,
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
