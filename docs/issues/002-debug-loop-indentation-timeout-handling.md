# Bug: Debug loop dispatched zeroed DEBUG_EVENT on timeout

**Date:** 2026-05-28
**Branch:** `001-debugger-engine`
**Commit:** `4a65294` (fixed as part of the debug-loop rewrite)

## Symptom

When `WaitForDebugEvent` timed out (100 ms), the debug log was flooded with:

```
ContinueDebugEvent failed: error=WIN32_ERROR(6)   // ERROR_INVALID_HANDLE
```

Even though no debug event had actually occurred.

## Root Cause

The code inside `debug_loop.rs` had an **indentation bug**. After the
`if result.is_ok()` block, the `dispatch_event` and `ContinueDebugEvent` calls
were syntactically **outside** the `if`, so they ran on *every* iteration of the
loop — including when `WaitForDebugEvent` had timed out and the `DEBUG_EVENT`
struct was still zeroed/undefined:

```rust
// BEFORE (buggy)
let result = unsafe { WaitForDebugEvent(&mut event, 100) };
if result.is_ok() {
    info!("Debug event: ...");
} else {
    // timeout — ignore
}
    // BUG: these lines were indented but NOT inside the if block
    let needs_continue = dispatch_event(&event, &event_tx);
    if needs_continue {
        ContinueDebugEvent(event.dwProcessId, event.dwThreadId, ...);
    }
```

On timeout:
- `event.dwProcessId` == 0, `event.dwThreadId` == 0
- `dispatch_event` sent a bogus `ProcessLaunched { pid: 0 }`
- `ContinueDebugEvent(0, 0, ...)` returned `ERROR_INVALID_HANDLE`

## Fix

The debug loop was completely rewritten with a `match` expression so the
dispatch code only runs when `WaitForDebugEvent` succeeds:

```rust
// AFTER (fixed)
match unsafe { WaitForDebugEventEx(&mut event, 100) } {
    Ok(()) => {
        let needs_continue = dispatch_event(&event, &event_tx);
        if needs_continue { ... }
    }
    Err(e) => {
        let code = e.code().0 as u32;
        if code != 0x00000102 && code != 0x00000079 {
            warn!("WaitForDebugEventEx failed: error=0x{:08X}", code);
        }
    }
}
```

## Bonus fix: GetLastError() staleness

The original error-handling code called `GetLastError()` manually after the
function returned `Err`:

```rust
let err = unsafe { GetLastError() };
```

Because `GetLastError()` is thread-local, this could read a stale error from a
different API call on the same thread. The fix uses the error code embedded in
the `windows::core::Result` instead:

```rust
Err(e) => {
    let code = e.code().0 as u32;  // reliable, captured by the wrapper
}
```
