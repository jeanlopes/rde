//! Debug event loop using WaitForDebugEventEx.

use rde_core::{DebugLoopCommand, EngineEvent, ProcessHandle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use windows::Win32::System::Diagnostics::Debug::{
    ContinueDebugEvent, WaitForDebugEventEx, DEBUG_EVENT,
    EXCEPTION_DEBUG_EVENT, EXIT_PROCESS_DEBUG_EVENT, CREATE_THREAD_DEBUG_EVENT,
    EXIT_THREAD_DEBUG_EVENT, LOAD_DLL_DEBUG_EVENT, CREATE_PROCESS_DEBUG_EVENT,
};

use windows::Win32::Foundation::NTSTATUS;

const DBG_CONTINUE: NTSTATUS = NTSTATUS(0x00010002);
const DBG_EXCEPTION_NOT_HANDLED: NTSTATUS = NTSTATUS(0x80010001_u32 as i32);
const STATUS_BREAKPOINT: u32 = 0x80000003;
const STATUS_SINGLE_STEP: u32 = 0x80000004;

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
    mut command_rx: mpsc::UnboundedReceiver<DebugLoopCommand>,
) -> DebugLoopControl {
    let control = DebugLoopControl::new();
    let running = control.running.clone();

    std::thread::spawn(move || {
        info!("Debug loop thread started");
        while running.load(Ordering::SeqCst) {
            let mut event = DEBUG_EVENT::default();
            // SAFETY: WaitForDebugEventEx writes into a valid DEBUG_EVENT struct on the stack.
            // Timeout of 100ms allows periodic checks of the running flag.
            let result = unsafe { WaitForDebugEventEx(&mut event, 100) };
            if result.is_ok() {
                let needs_continue = dispatch_event(&event, &event_tx);

                if needs_continue {
                    // For events that pause the process, wait for the engine to tell us to continue.
                    match event.dwDebugEventCode {
                        CREATE_PROCESS_DEBUG_EVENT | EXCEPTION_DEBUG_EVENT => {
                            // Block until engine sends a continue command
                            match command_rx.blocking_recv() {
                                Some(DebugLoopCommand::Continue) => {
                                    let status = if event.dwDebugEventCode == EXCEPTION_DEBUG_EVENT {
                                        let code = unsafe { event.u.Exception.ExceptionRecord.ExceptionCode.0 as u32 };
                                        if code == STATUS_BREAKPOINT || code == STATUS_SINGLE_STEP {
                                            DBG_CONTINUE
                                        } else {
                                            DBG_EXCEPTION_NOT_HANDLED
                                        }
                                    } else {
                                        DBG_CONTINUE
                                    };
                                    let _ = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, status) };
                                }
                                Some(DebugLoopCommand::ContinueException) => {
                                    let _ = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, DBG_EXCEPTION_NOT_HANDLED) };
                                }
                                None => {
                                    // Channel closed, exit loop
                                    break;
                                }
                            }
                        }
                        _ => {
                            // Other events: continue automatically
                            let _ = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, DBG_CONTINUE) };
                        }
                    }
                } else {
                    // Event that doesn't need explicit continue (shouldn't happen with WaitForDebugEventEx)
                    let _ = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, DBG_CONTINUE) };
                }
            }
        }
        info!("Debug loop thread exiting");
    });

    control
}

/// Dispatch a debug event. Returns true if the event requires a ContinueDebugEvent call.
fn dispatch_event(
    event: &DEBUG_EVENT,
    tx: &mpsc::UnboundedSender<EngineEvent>,
) -> bool {
    match event.dwDebugEventCode {
        CREATE_PROCESS_DEBUG_EVENT => {
            info!("Process created: PID {}", event.dwProcessId);
            let _ = tx.send(EngineEvent::ProcessLaunched {
                pid: event.dwProcessId,
            });
            true
        }
        EXIT_PROCESS_DEBUG_EVENT => {
            let exit_code = unsafe { event.u.ExitProcess.dwExitCode };
            info!("Process exited: PID {} code {}", event.dwProcessId, exit_code);
            let _ = tx.send(EngineEvent::ProcessExited { code: exit_code });
            true
        }
        CREATE_THREAD_DEBUG_EVENT => {
            let tid = event.dwThreadId;
            info!("Thread created: TID {}", tid);
            let _ = tx.send(EngineEvent::ThreadCreated { id: tid });
            true
        }
        EXIT_THREAD_DEBUG_EVENT => {
            let tid = event.dwThreadId;
            let exit_code = unsafe { event.u.ExitThread.dwExitCode };
            info!("Thread exited: TID {} code {}", tid, exit_code);
            let _ = tx.send(EngineEvent::ThreadExited {
                id: tid,
                code: exit_code,
            });
            true
        }
        LOAD_DLL_DEBUG_EVENT => {
            let _ = tx.send(EngineEvent::ModuleLoaded {
                name: "unknown".into(),
                base: 0,
            });
            true
        }
        EXCEPTION_DEBUG_EVENT => {
            let exception = unsafe { &event.u.Exception };
            let code = exception.ExceptionRecord.ExceptionCode.0 as u32;
            let address = exception.ExceptionRecord.ExceptionAddress as u64;
            info!("Exception: code 0x{:08X} at 0x{:X}", code, address);

            if code == STATUS_BREAKPOINT {
                let _ = tx.send(EngineEvent::BreakpointHit {
                    id: 0, // Engine resolves breakpoint ID from address
                    address,
                    thread_id: event.dwThreadId,
                });
            } else if code == STATUS_SINGLE_STEP {
                let _ = tx.send(EngineEvent::SingleStep {
                    address,
                    thread_id: event.dwThreadId,
                });
            } else {
                let _ = tx.send(EngineEvent::Exception { code, address });
            }
            true
        }
        _ => true,
    }
}
