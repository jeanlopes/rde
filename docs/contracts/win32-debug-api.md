# Win32 Debug API Contracts

> This document records the pre-conditions, post-conditions, and invariants of every Win32 API used by the RDE Windows backend (`rde-win32`).
> **Rule:** Before modifying any code in `debug_loop.rs`, `process.rs`, or `engine.rs`, verify that the change respects these contracts.

---

## Contract: `CreateProcessW` with `DEBUG_PROCESS`

### Pre-conditions
- Valid wide-string command line (`PWSTR`).
- `STARTUPINFOW` struct initialized with `cb = sizeof(STARTUPINFOW)`.

### Post-conditions (Ok)
- Process is created and suspended until the debugger's debug loop begins waiting.
- The thread that called `CreateProcessW` becomes the *debugging thread* for this process.
- `PROCESS_INFORMATION` contains valid `hProcess` and `hThread` handles.

### Invariant — THREAD AFFINITY
- The thread that called `CreateProcessW` with `DEBUG_PROCESS` **MUST** be the same thread that later calls `WaitForDebugEventEx`.
- Violation: `WaitForDebugEventEx` returns `ERROR_INVALID_HANDLE` or hangs silently.
- **RDE enforcement:** `launch_sync` is called from inside the debug loop thread spawned by `WindowsBackend::launch`.

---

## Contract: `WaitForDebugEventEx`

### Pre-conditions
- Called from the same thread that created the process with `DEBUG_PROCESS`.
- `DEBUG_EVENT` struct is writable memory.
- Timeout is in milliseconds (`dwMilliseconds`).

### Post-conditions (Ok)
- `DEBUG_EVENT` is **fully populated** and valid.
- The debuggee process and the specific thread reported are **suspended**.
- The debugger **MUST** eventually call `ContinueDebugEvent` with the exact `dwProcessId` and `dwThreadId` from this struct.

### Post-conditions (Err)
- `DEBUG_EVENT` struct is **UNDEFINED**.
  - May be zeroed.
  - May contain garbage.
  - **MUST NOT be read.**
- **MUST NOT** call `dispatch_event` with this struct.
- **MUST NOT** call `ContinueDebugEvent` with IDs from this struct.

### Timeout Handling
- `ERROR_SEM_TIMEOUT` (`0x102`): Normal. Loop back to `WaitForDebugEventEx`.
- After `EXIT_PROCESS_DEBUG_EVENT`: may receive `0x80070079` (HRESULT from `ERROR_SEM_TIMEOUT`). Also normal. Exit the loop.

### RDE Invariant
```rust
// INVARIANT: event is ONLY valid when WaitForDebugEventEx returns Ok(()).
match unsafe { WaitForDebugEventEx(&mut event, 100) } {
    Ok(()) => {
        // SAFE: event is populated. May read dwDebugEventCode, call dispatch_event,
        //       and later call ContinueDebugEvent with event.dwProcessId / dwThreadId.
    }
    Err(e) => {
        // SAFE: event is UNINITIALIZED. Do NOT read it. Do NOT call dispatch_event.
        //       Do NOT call ContinueDebugEvent.
    }
}
```

---

## Contract: `ContinueDebugEvent`

### Pre-conditions
- `dwProcessId` and `dwThreadId` **must** come from a `DEBUG_EVENT` that was returned by a successful `WaitForDebugEventEx` call.
- Must be called exactly once per successful debug event (except `EXIT_PROCESS_DEBUG_EVENT`, after which the loop exits).

### Status Codes
- `DBG_CONTINUE` (`0x00010001`): Exception is handled. The thread continues execution.
  - Use for: process/thread creation, DLL loads, **system breakpoints**, and **our own breakpoints** when we have restored state.
- `DBG_EXCEPTION_NOT_HANDLED` (`0x80010001`): Exception is not handled by the debugger. Passes to the debuggee's exception handler.
  - Use for: unknown / unexpected exceptions that the program should handle itself.

### Invariant — System Initial Breakpoint
- Windows inserts an `int3` in `ntdll` before the process entry point.
- This is **NOT** a user-defined breakpoint.
- **Rule:** `DBG_CONTINUE` only. Do NOT decrement RIP. Do NOT set Trap Flag. Do NOT restore any original byte.
- Violation: Infinite breakpoint→single-step→breakpoint loop because the `int3` byte is never restored (it was never ours to restore).

---

## Contract: `DebugActiveProcess`

### Pre-conditions
- Target process is not already being debugged.
- Caller has `SE_DEBUG_NAME` privilege (or is running as admin / same user).

### Post-conditions (Ok)
- Debugger attaches to the target process.
- The attaching thread becomes the debugging thread (same thread-affinity rule as `CreateProcessW`).
- A `CREATE_PROCESS_DEBUG_EVENT` is generated for the debugger.

---

## Quick Reference: Error Codes

| Code | Name | Meaning | Action |
|------|------|---------|--------|
| `0x06` | `ERROR_INVALID_HANDLE` | Handle used when invalid | Check if `DEBUG_EVENT` was read after timeout |
| `0x102` | `ERROR_SEM_TIMEOUT` | Normal timeout from `WaitForDebugEventEx` | Ignore, loop back |
| `0x80070079` | `HRESULT_FROM_WIN32(ERROR_SEM_TIMEOUT)` | Post-exit timeout | Ignore, exit loop |
| `0x05` | `ERROR_ACCESS_DENIED` | Permissions or process already debugged | Verify privileges / not double-debugging |
