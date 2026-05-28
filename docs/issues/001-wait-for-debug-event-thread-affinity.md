# Bug: WaitForDebugEvent fails with ERROR_INVALID_HANDLE in separate thread

**Date:** 2026-05-28
**Branch:** `001-debugger-engine`
**Commit:** `4a65294`

## Symptom

After launching a process with `CreateProcessW` + `DEBUG_PROCESS`, the debug loop
thread would immediately and repeatedly fail with:

```
WaitForDebugEvent failed: error=0x80070006  // ERROR_INVALID_HANDLE
```

The debuggee process received a PID but never executed — it remained suspended
because no debug events could be consumed.

## Root Cause

On Windows, the thread that calls **`CreateProcessW` with `DEBUG_PROCESS` (or
`DEBUG_ONLY_THIS_PROCESS`) must also be the thread that calls `WaitForDebugEvent`**
(or `WaitForDebugEventEx`). The OS internally associates the debug port with the
calling thread; if another thread tries to wait for events, `WaitForDebugEvent`
returns `ERROR_INVALID_HANDLE`.

Our original architecture was:

1. Async Tokio task → `backend.launch()` → `CreateProcessW` (creates the process)
2. Later, `backend.on_session_started()` → `std::thread::spawn` → debug loop
   that calls `WaitForDebugEvent`

Because the debug loop ran on a **different thread** from the one that created the
process, `WaitForDebugEvent` always failed.

## Evidence

A minimal standalone repro confirmed the behaviour:

| Scenario | Result |
|---|---|
| `CreateProcessW` + `WaitForDebugEvent` on **same** thread | ✅ Works |
| `CreateProcessW` on main thread, `WaitForDebugEvent` on **spawned** thread | ❌ `ERROR_INVALID_HANDLE` |

## Fix

Move process creation **into** the debug-loop thread so both APIs run on the
same thread:

```rust
// rde-win32/src/lib.rs
async fn launch(&self, path: &Path, args: &[String]) -> Result<(ProcessHandle, DebugChannels), DebugError> {
    let (handle_tx, handle_rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(move || {
        match process::launch(&path, &args) {
            Ok(handle) => {
                let _ = handle_tx.send(handle.clone());
                debug_loop::run_debug_loop(&handle, event_tx, command_rx);
            }
            Err(e) => { /* handle_tx dropped → receiver gets error */ }
        }
    });

    let handle = handle_rx.await?;
    Ok((handle, (event_rx, debug_loop_tx)))
}
```

### Interface changes

- `DebugBackend::launch` now returns `(ProcessHandle, DebugChannels)` instead of
  just `ProcessHandle`.
- `DebugBackend::attach` was updated with the same signature.
- `on_session_started` was removed from the trait; the backend now fully owns
  debug-loop lifecycle.
- The engine replaces its internal `debug_event_rx` with the receiver returned
  by `launch()`.

### Secondary fixes in the same commit

- The debug loop now exits cleanly after `EXIT_PROCESS_DEBUG_EVENT` instead of
  looping forever on `ERROR_SEM_TIMEOUT` (`0x79`).
- `0x79` was added to the ignored-timeout filter alongside the existing `0x102`.

## References

- Microsoft Docs: [`WaitForDebugEvent` function](https://learn.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-waitfordebugevent)
  (documentation *claims* the function can be called from any thread, but
  empirical evidence shows the thread that created the debugged process is
  required when `DEBUG_PROCESS` is used at creation time).
