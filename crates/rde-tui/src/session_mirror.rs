//! Cached view of debug session state, updated from EngineEvents.

use rde_core::{Breakpoint, DisassemblyLine, EngineEvent, RegisterContext, StackFrame, Thread};

/// Aggregated debug session state mirrored from EngineEvents.
#[derive(Debug, Clone, Default)]
pub struct SessionMirror {
    pub state: SessionState,
    pub selected_thread: Option<u32>,
    pub threads: Vec<Thread>,
    pub breakpoints: Vec<Breakpoint>,
    pub registers: Option<RegisterContext>,
    pub disassembly: Vec<DisassemblyLine>,
    pub stack_trace: Vec<StackFrame>,
    pub current_address: Option<u64>,
    pub repl_history: Vec<(String, String)>,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Default)]
pub enum SessionState {
    #[default]
    Running,
    Paused,
    Exited,
}


impl SessionMirror {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, event: &EngineEvent) {
        match event {
            EngineEvent::ProcessLaunched { .. } | EngineEvent::ProcessAttached { .. } => {
                self.state = SessionState::Running;
            }
            EngineEvent::BreakpointHit { address, .. } => {
                self.state = SessionState::Paused;
                self.current_address = Some(*address);
            }
            EngineEvent::SingleStep { address, .. } => {
                self.state = SessionState::Paused;
                self.current_address = Some(*address);
            }
            EngineEvent::ProcessExited { .. } => {
                self.state = SessionState::Exited;
            }
            EngineEvent::Exception { .. } => {
                self.state = SessionState::Paused;
            }
            EngineEvent::Registers { ctx } => {
                self.registers = Some(ctx.clone());
            }
            EngineEvent::Disassembly { lines } => {
                self.disassembly = lines.clone();
            }
            EngineEvent::BreakpointList { list } => {
                self.breakpoints = list.clone();
            }
            EngineEvent::StackTrace { frames } => {
                self.stack_trace = frames.clone();
            }
            EngineEvent::ThreadCreated { id, handle } => {
                if !self.threads.iter().any(|t| t.id == *id) {
                    self.threads.push(Thread {
                        id: *id,
                        handle: rde_core::RawHandle(*handle),
                        state: rde_core::ThreadState::Running,
                        context: None,
                    });
                }
            }
            EngineEvent::ThreadExited { id, .. } => {
                if let Some(t) = self.threads.iter_mut().find(|t| t.id == *id) {
                    t.state = rde_core::ThreadState::Exited { exit_code: 0 };
                }
            }
            _ => {}
        }
    }
}
