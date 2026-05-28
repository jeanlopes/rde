//! Debug event loop using WaitForDebugEventEx.

use rde_core::{EngineCommand, EngineEvent, ProcessHandle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use windows::Win32::System::Diagnostics::Debug::{
    ContinueDebugEvent, WaitForDebugEventEx, DEBUG_EVENT,
    EXCEPTION_DEBUG_EVENT, EXIT_PROCESS_DEBUG_EVENT, CREATE_THREAD_DEBUG_EVENT,
    EXIT_THREAD_DEBUG_EVENT, LOAD_DLL_DEBUG_EVENT, CREATE_PROCESS_DEBUG_EVENT,
};

use windows::Win32::Foundation::NTSTATUS;

const DBG_CONTINUE: NTSTATUS = NTSTATUS(0x00010002);
const DBG_EXCEPTION_NOT_HANDLED: NTSTATUS = NTSTATUS(0x80010001_u32 as i32);
const STATUS_BREAKPOINT: u32 = 0x80000003;

/// Shared state to control the debug loop.
pub struct DebugLoopControl {
    pub running: Arc<AtomicBool>,
}

impl DebugLoopControl {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Start the debug event loop in a dedicated std::thread.
///
/// Returns a `DebugLoopControl` handle to stop the loop gracefully.
pub fn start_debug_loop(
    _handle: &ProcessHandle,
    event_tx: mpsc::UnboundedSender<EngineEvent>,
    _command_rx: mpsc::UnboundedReceiver<EngineCommand>,
) -> DebugLoopControl {
    let control = DebugLoopControl::new();
    let running = control.running.clone();

    std::thread::spawn(move || {
        info!("Debug loop thread started");
        while running.load(Ordering::SeqCst) {
            let mut event = DEBUG_EVENT::default();
            // SAFETY: WaitForDebugEventEx writes into a valid DEBUG_EVENT struct on the stack.
            // Timeout of 100ms allows periodic checks of the running flag.
            // GitHub issue: TODO(rde#1)
            let result = unsafe { WaitForDebugEventEx(&mut event, 100) };
            if result.is_ok() {
                if let Err(e) = dispatch_event(&event, &event_tx) {
                    error!("Failed to dispatch debug event: {e}");
                }
                let status: NTSTATUS = if event.dwDebugEventCode == EXCEPTION_DEBUG_EVENT {
                    DBG_CONTINUE
                } else {
                    DBG_EXCEPTION_NOT_HANDLED
                };
                // SAFETY: dwProcessId and dwThreadId are valid IDs from the event.
                let _ = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, status) };
            }
        }
        info!("Debug loop thread exiting");
    });

    control
}

fn dispatch_event(
    event: &DEBUG_EVENT,
    tx: &mpsc::UnboundedSender<EngineEvent>,
) -> Result<(), String> {
    match event.dwDebugEventCode {
        CREATE_PROCESS_DEBUG_EVENT => {
            info!("Process created: PID {}", event.dwProcessId);
            let _ = tx.send(EngineEvent::ProcessLaunched {
                pid: event.dwProcessId,
            });
        }
        EXIT_PROCESS_DEBUG_EVENT => {
            let exit_code = unsafe { event.u.ExitProcess.dwExitCode };
            info!("Process exited: PID {} code {}", event.dwProcessId, exit_code);
            let _ = tx.send(EngineEvent::ProcessExited { code: exit_code });
        }
        CREATE_THREAD_DEBUG_EVENT => {
            let tid = event.dwThreadId;
            info!("Thread created: TID {}", tid);
            let _ = tx.send(EngineEvent::ThreadCreated { id: tid });
        }
        EXIT_THREAD_DEBUG_EVENT => {
            let tid = event.dwThreadId;
            let exit_code = unsafe { event.u.ExitThread.dwExitCode };
            info!("Thread exited: TID {} code {}", tid, exit_code);
            let _ = tx.send(EngineEvent::ThreadExited {
                id: tid,
                code: exit_code,
            });
        }
        LOAD_DLL_DEBUG_EVENT => {
            let _ = tx.send(EngineEvent::ModuleLoaded {
                name: "unknown".into(),
                base: 0,
            });
        }
        EXCEPTION_DEBUG_EVENT => {
            let exception = unsafe { &event.u.Exception };
            let code = exception.ExceptionRecord.ExceptionCode.0 as u32;
            let address = exception.ExceptionRecord.ExceptionAddress as u64;
            info!("Exception: code 0x{:08X} at 0x{:X}", code, address);

            if code == STATUS_BREAKPOINT {
                let _ = tx.send(EngineEvent::BreakpointHit {
                    id: 0, // TODO: resolve breakpoint ID from address
                    address,
                    thread_id: event.dwThreadId,
                });
            } else {
                let _ = tx.send(EngineEvent::Exception { code, address });
            }
        }
        _ => {}
    }
    Ok(())
}
