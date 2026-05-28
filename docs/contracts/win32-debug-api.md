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

## Contract: `OpenThread`

### Pre-conditions
- Valid `ThreadId` exists in the target process.
- Desired access mask is valid (e.g., `THREAD_ALL_ACCESS` for debugging).

### Post-conditions (Ok)
- Returns a valid handle with the requested access rights.

### Post-conditions (Err)
- `ERROR_INVALID_PARAMETER`: TID does not exist or caller lacks privilege.

### Invariant
- The returned handle **MUST** be closed with `CloseHandle` to prevent handle leaks.
- RDE caches handles from `CREATE_THREAD_DEBUG_EVENT` and closes them on `EXIT_THREAD_DEBUG_EVENT`.

---

## Contract: `GetMappedFileNameW`

### Pre-conditions
- Process handle has `PROCESS_QUERY_INFORMATION`.
- `lpv` points to a valid mapped address in the target process.

### Post-conditions (Ok)
- Buffer contains the NT device path of the mapped file.
- Return value is the length in characters, excluding null terminator.

### Post-conditions (Err)
- Returns 0 on error (e.g., address not mapped).
- `GetLastError` may be set.

### Invariant
- The returned path is an NT path (`\Device\HarddiskVolume...`), NOT a DOS path.
- For display to users, translate to a DOS path via `QueryDosDevice`.

---

## Contract: `EnumProcessModules`

### Pre-conditions
- Process handle has `PROCESS_QUERY_INFORMATION | PROCESS_VM_READ`.

### Post-conditions (Ok)
- `lphModule` array populated with `HMODULE` handles.
- `lpcbNeeded` contains total bytes required.

### Post-conditions (Err)
- `ERROR_PARTIAL_COPY`: Cross-bitness process access.

### Invariant
- Snapshot API: modules loaded/unloaded after the call are not reflected.
- Use as fallback only; primary tracking is event-driven via `LOAD_DLL_DEBUG_EVENT`.

---

## Contract: `SymLoadModuleExW`

### Pre-conditions
- `SymInitializeW` called for the process handle.
- `BaseOfDll` is the load address from `LOAD_DLL_DEBUG_EVENT`.

### Post-conditions (Ok)
- Module registered with DbgHelp; symbol resolution works for addresses in the module.
- Returns non-zero module base.

### Post-conditions (Err)
- Returns 0 on failure. `GetLastError` may be `ERROR_NOT_SUPPORTED` if no PDB found.
- Existing symbol table is NOT corrupted.

### Invariant
- Must be called for every module loaded AFTER `SymInitializeW`.
- Calling for an already-loaded module is a no-op.
- Must be paired with `SymUnloadModule64` on module unload.

---

## Quick Reference: Error Codes

| Code | Name | Meaning | Action |
|------|------|---------|--------|
| `0x06` | `ERROR_INVALID_HANDLE` | Handle used when invalid | Check if `DEBUG_EVENT` was read after timeout |
| `0x102` | `ERROR_SEM_TIMEOUT` | Normal timeout from `WaitForDebugEventEx` | Ignore, loop back |
| `0x80070079` | `HRESULT_FROM_WIN32(ERROR_SEM_TIMEOUT)` | Post-exit timeout | Ignore, exit loop |
| `0x05` | `ERROR_ACCESS_DENIED` | Permissions or process already debugged | Verify privileges / not double-debugging |
