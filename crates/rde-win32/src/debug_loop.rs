//! Debug event loop using WaitForDebugEventEx.
//!
//! CRITICAL INVARIANT — THREAD AFFINITY:
//! On Windows, `CreateProcessW` with `DEBUG_PROCESS` and
//! `WaitForDebugEventEx` MUST run on the *same* thread.  Therefore this
//! function is called from the thread that also created the process.
//!
//! CRITICAL INVARIANT — DEBUG_EVENT VALIDITY:
//! The `DEBUG_EVENT` struct is ONLY valid when `WaitForDebugEventEx` returns
//! `Ok(())`. On `Err` (including timeouts), the struct is UNINITIALIZED and
//! MUST NOT be read or passed to `ContinueDebugEvent`.
//! Violation: ERROR_INVALID_HANDLE (0x06) spam.
//!
//! CRITICAL INVARIANT — SYSTEM INITIAL BREAKPOINT:
//! Windows inserts an `int3` in ntdll before the entry point. This is NOT a
//! user breakpoint. It must be continued with DBG_CONTINUE only — never
//! decrement RIP, never set Trap Flag, never restore original byte.
//! Violation: infinite breakpoint→single-step→breakpoint loop.

use rde_core::{DebugLoopCommand, EngineEvent, ProcessHandle};
use tokio::sync::mpsc;
use tracing::{info, warn};
use windows::Win32::System::Diagnostics::Debug::{
    ContinueDebugEvent, WaitForDebugEventEx, DEBUG_EVENT,
    EXCEPTION_DEBUG_EVENT, EXIT_PROCESS_DEBUG_EVENT, CREATE_THREAD_DEBUG_EVENT,
    EXIT_THREAD_DEBUG_EVENT, LOAD_DLL_DEBUG_EVENT, CREATE_PROCESS_DEBUG_EVENT,
};

use windows::Win32::Foundation::NTSTATUS;

const DBG_CONTINUE: NTSTATUS = NTSTATUS(0x00010001);
const DBG_EXCEPTION_NOT_HANDLED: NTSTATUS = NTSTATUS(0x80010001_u32 as i32);
const STATUS_BREAKPOINT: u32 = 0x80000003;
const STATUS_SINGLE_STEP: u32 = 0x80000004;

/// Run the debug event loop on the *current* thread.
///
/// This function blocks until the debuggee exits or the command channel closes.
pub fn run_debug_loop(
    _handle: &ProcessHandle,
    event_tx: mpsc::UnboundedSender<EngineEvent>,
    mut command_rx: mpsc::UnboundedReceiver<DebugLoopCommand>,
) {
    info!("Debug loop started on thread {:?}", std::thread::current().id());
    let mut process_running = true;
    while process_running {
        let mut event = DEBUG_EVENT::default();
        // INVARIANT: event is ONLY valid when WaitForDebugEventEx returns Ok(()).
        match unsafe { WaitForDebugEventEx(&mut event, 100) } {
            Ok(()) => {
                // SAFE: event is fully populated. We may read dwDebugEventCode,
                // call dispatch_event, and later call ContinueDebugEvent with
                // event.dwProcessId / dwThreadId.
                info!(
                    target: "rde::debug_loop::state",
                    code = ?event.dwDebugEventCode,
                    pid = event.dwProcessId,
                    tid = event.dwThreadId,
                    "WaitForDebugEventEx returned Ok"
                );

                let needs_continue = dispatch_event(&event, &event_tx);

                if event.dwDebugEventCode == EXIT_PROCESS_DEBUG_EVENT {
                    process_running = false;
                }

                if needs_continue {
                    match event.dwDebugEventCode {
                        CREATE_PROCESS_DEBUG_EVENT | EXCEPTION_DEBUG_EVENT => {
                            info!("Waiting for engine command to continue...");
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
                                    info!("Calling ContinueDebugEvent with status=0x{:08X}", status.0);
                                    let cont_result = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, status) };
                                    if let Err(e) = cont_result {
                                        warn!("ContinueDebugEvent failed: {:?}", e);
                                    }
                                }
                                Some(DebugLoopCommand::ContinueException) => {
                                    info!("Calling ContinueDebugEvent with DBG_EXCEPTION_NOT_HANDLED");
                                    let cont_result = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, DBG_EXCEPTION_NOT_HANDLED) };
                                    if let Err(e) = cont_result {
                                        warn!("ContinueDebugEvent failed: {:?}", e);
                                    }
                                }
                                None => {
                                    info!("Command channel closed, exiting debug loop");
                                    break;
                                }
                            }
                        }
                        _ => {
                            let cont_result = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, DBG_CONTINUE) };
                            if let Err(e) = cont_result {
                                warn!("ContinueDebugEvent failed: {:?}", e);
                            }
                        }
                    }
                } else {
                    let cont_result = unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, DBG_CONTINUE) };
                    if let Err(e) = cont_result {
                        warn!("ContinueDebugEvent failed: {:?}", e);
                    }
                }
            }
            Err(e) => {
                // SAFE: event is UNINITIALIZED here. Do NOT read it.
                // Do NOT call dispatch_event. Do NOT call ContinueDebugEvent.
                let code = e.code().0 as u32;
                if code == 0x00000102 {
                    // ERROR_SEM_TIMEOUT — normal timeout, ignore silently
                } else if code == 0x80070079 {
                    // Post-exit timeout after process death, ignore silently
                } else {
                    warn!(
                        target: "rde::debug_loop::state",
                        error_code = format!("0x{:08X}", code),
                        "WaitForDebugEventEx returned Err"
                    );
                }
            }
        }
    }
    info!("Debug loop exiting");
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
                // The debug loop does not know if this is a system breakpoint
                // or a user-defined one. Send Unknown and let the engine resolve.
                let _ = tx.send(EngineEvent::BreakpointHit {
                    kind: rde_core::BreakpointKind::Unknown,
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
