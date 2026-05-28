# Bug: Unknown breakpoints caused infinite breakpoint→single-step loop

**Date:** 2026-05-28
**Branch:** `001-debugger-engine`
**Commit:** `4a65294` (fixed in engine.rs)

## Symptom

When launching any debuggee, the process would hit the initial system
breakpoint (inserted by Windows in `ntdll` before the entry point) and then
enter an infinite loop:

```
breakpoint hit → single-step → breakpoint hit → single-step → ...
```

The debuggee never executed user code and produced no output.

## Root Cause

The engine's `BreakpointHit` handler **unconditionally** performed the
restore-step-reinstall dance for *every* breakpoint, including unknown ones:

```rust
// BEFORE (buggy)
EngineEvent::BreakpointHit { id, address, thread_id } => {
    // 1. Restore original byte (only if known breakpoint)
    if let Some(bp) = self.breakpoints.get(id) {
        self.backend.write_memory(..., &[bp.original_byte]).await;
    }
    // 2. Decrement RIP — BUG: done even for UNKNOWN breakpoints
    let mut ctx = self.backend.get_registers(...).await?;
    ctx.rip = address;
    // 3. Set Trap Flag — BUG: same issue
    ctx.rflags |= 0x100;
    self.backend.set_registers(..., &ctx).await?;
    self.stepping_over_breakpoint = Some(address);
    // 4. Auto-continue — triggers the infinite loop
    tx.send(DebugLoopCommand::Continue);
}
```

For the **initial system breakpoint** (and any other unknown breakpoint):
- The original byte at the hit address was **never restored** (because `id == 0`
  and `breakpoints.get(0)` returned `None`).
- Memory still contained `0xCC` (the `int3` instruction).
- RIP was decremented back to the same address.
- Trap Flag was set, causing a single-step exception.
- After the single-step, the engine tried to reinstall a breakpoint at the
  address — but since it was unknown, nothing happened.
- The CPU executed the `0xCC` at that address again → another breakpoint hit.

## Fix

The handler now checks if the breakpoint is unknown (`id == 0`) and, if so,
**reports the event and continues without touching registers or memory**:

```rust
// AFTER (fixed)
EngineEvent::BreakpointHit { id, address, thread_id } => {
    let id = if id == 0 {
        self.breakpoints.find_by_address(address).map(|bp| bp.id).unwrap_or(0)
    } else { id };

    if id == 0 {
        // Unknown breakpoint (e.g., Windows initial system breakpoint)
        info!("Unknown breakpoint, sending Continue to debug loop");
        let _ = self.event_tx.send(EngineEvent::BreakpointHit { id: 0, address, thread_id });
        if let Some(tx) = &self.debug_loop_tx {
            let _ = tx.send(DebugLoopCommand::Continue);
        }
        return Ok(());
    }

    // Known breakpoint: safe to restore byte, decrement RIP, set TF, etc.
    ...
}
```

## Tested

`rust_app_example.exe --demo` now:
1. Hits the initial system breakpoint at `0x7ffeb89bf58d`
2. Engine detects `id == 0`, reports it, and continues
3. Debuggee runs to completion and prints all demo output
4. Exits cleanly with code 0
