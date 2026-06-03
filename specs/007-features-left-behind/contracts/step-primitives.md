# Win32 Contract: Step-Over and Step-Out Primitives

**Feature**: 007-features-left-behind
**Crate**: `rde-win32/src/step.rs` (new file)
**Date**: 2026-06-01

> These contracts MUST be ported to `docs/contracts/win32-debug-api.md` before merging.

---

## 1. `GetThreadContext` (used by both primitives)

**Purpose**: Read current register state (RIP, RSP, RBP) while the target thread is suspended.

| Attribute | Value |
|-----------|-------|
| Header | `windows::Win32::System::Diagnostics::Debug` |
| Signature | `GetThreadContext(hThread: HANDLE, lpContext: *mut CONTEXT) -> BOOL` |
| Thread affinity | Must be called from the debug-loop thread (the thread that called `WaitForDebugEventEx`) |

**Pre-conditions**:
1. `hThread` is a valid handle obtained via `OpenThread(THREAD_ALL_ACCESS, ...)` for the thread whose ID was reported in the last `DEBUG_EVENT`.
2. The target thread is **suspended** — this is guaranteed inside a `WaitForDebugEventEx` handler before `ContinueDebugEvent` is called.
3. `lpContext` points to a properly initialized `CONTEXT` with `ContextFlags = CONTEXT_ALL_AMD64`.

**Post-conditions (Ok / `BOOL == TRUE`)**:
- `lpContext.Rip` = instruction pointer at time of suspension.
- `lpContext.Rsp` = stack pointer at time of suspension.
- `lpContext.Rbp` = frame pointer at time of suspension.
- All other `CONTEXT` fields valid per `CONTEXT_ALL_AMD64`.

**Post-conditions (Err / `BOOL == FALSE`)**:
- `lpContext` fields are **undefined** — do NOT read any register value.
- Call `GetLastError()` immediately; do not issue further Win32 calls before capturing the error code.

**Invariants**:
```rust
// INVARIANT: GetThreadContext returns undefined data if the thread is not suspended.
// VIOLATION: Reading RIP from a running thread produces a garbage/stale value that will
//            plant a temp BP at a wrong address, corrupting execution state.
//            See issue: TODO(rde#step-over-contract)
```

---

## 2. `ReadProcessMemory` (read return address for step-out; read instruction bytes for Capstone)

**Purpose**: Read bytes from the target process's virtual address space.

| Attribute | Value |
|-----------|-------|
| Header | `windows::Win32::System::Diagnostics::Debug` |
| Signature | `ReadProcessMemory(hProcess, lpBaseAddress, lpBuffer, nSize, lpNumberOfBytesRead) -> BOOL` |

**Pre-conditions**:
1. `hProcess` is the process handle from `ProcessHandle::handle` (stored in `Session`).
2. `lpBaseAddress` is a valid virtual address in the target process — for step-out: `RSP`; for instruction decode: `RIP`.
3. `nSize` ≤ 16 for instruction bytes; exactly 8 for return address on x64.
4. The target page is readable (code pages: always; stack pages: always for the default thread stack).

**Post-conditions (Ok)**:
- `lpBuffer[0..nSize]` contains the bytes at `lpBaseAddress`.
- `*lpNumberOfBytesRead == nSize` (partial reads allowed by Win32 — check this!).

**Post-conditions (Err)**:
- `lpBuffer` contents are **undefined**.
- Common errors: `ERROR_PARTIAL_COPY` (page boundary or guard page), `ERROR_NOACCESS`.

**Invariants**:
```rust
// INVARIANT: Partial reads are legal — always check *lpNumberOfBytesRead == requested.
// VIOLATION: Interpreting a partial read as a full instruction length causes Capstone to
//            decode garbage, planting the temp BP at the wrong offset.
//            See issue: TODO(rde#step-over-contract)
```

---

## 3. `WriteProcessMemory` (plant / restore INT3 byte)

**Purpose**: Write the INT3 opcode (0xCC) to plant a temp BP; write the original byte to restore it.

| Attribute | Value |
|-----------|-------|
| Header | `windows::Win32::System::Diagnostics::Debug` |
| Signature | `WriteProcessMemory(hProcess, lpBaseAddress, lpBuffer, nSize, lpNumberOfBytesWritten) -> BOOL` |

**Pre-conditions**:
1. `hProcess` is a valid process handle with `PROCESS_VM_WRITE` and `PROCESS_VM_OPERATION` access.
2. `lpBaseAddress` is a valid writable virtual address.  
   ⚠️ Code pages are typically read-only in user mode — Win32 debug API process handles implicitly grant write access to code pages of the debuggee; this is a debugging-specific privilege.
3. `nSize == 1` (single byte write).

**Post-conditions (Ok)**:
- `*lpBaseAddress == lpBuffer[0]`.
- Instruction cache may be stale — **must** call `FlushInstructionCache` after writing INT3.

**Post-conditions (Err)**:
- Byte at `*lpBaseAddress` is **unchanged** — do NOT set `stepping_over_breakpoint`; abort the step.

**Invariants**:
```rust
// INVARIANT: Always call FlushInstructionCache after WriteProcessMemory on a code page.
// VIOLATION: Without cache flush, the CPU may execute the cached (non-patched) instruction,
//            causing the temp BP to never fire.
//            See issue: TODO(rde#step-over-contract)

// INVARIANT: Save the original byte BEFORE calling WriteProcessMemory(0xCC).
// VIOLATION: If WriteProcessMemory succeeds but we did not save the original byte, restoration
//            is impossible — the address is permanently INT3 until the process dies.
```

---

## 4. `FlushInstructionCache`

**Purpose**: Invalidate the CPU instruction cache for the range containing the modified byte.

| Attribute | Value |
|-----------|-------|
| Header | `windows::Win32::System::Diagnostics::Debug` |
| Signature | `FlushInstructionCache(hProcess, lpBaseAddress, dwSize) -> BOOL` |

**Pre-conditions**:
- Called immediately after any `WriteProcessMemory` on a code page.
- `dwSize == 1` sufficient for a single INT3 write.

**Post-conditions (Ok)**:
- CPU will fetch fresh bytes from memory on next execute at `lpBaseAddress`.

**Post-conditions (Err)**:
- `GetLastError()` returns the code; the write is already done — log and continue (best-effort).

**Invariants**:
```rust
// INVARIANT: Call FlushInstructionCache after every WriteProcessMemory on code pages.
// This is especially critical on multi-core systems where instruction cache is per-core.
```

---

## 5. `ContinueDebugEvent` (resume after planting temp BP)

| Attribute | Value |
|-----------|-------|
| Signature | `ContinueDebugEvent(dwProcessId, dwThreadId, dwContinueStatus) -> BOOL` |
| `dwContinueStatus` | Always `DBG_CONTINUE` for temp BP events (not `DBG_EXCEPTION_NOT_HANDLED`) |

**Pre-conditions**:
1. Called from the debug-loop thread.
2. `dwProcessId` and `dwThreadId` match the IDs from the most recent `WaitForDebugEventEx`.
3. Called exactly once per `WaitForDebugEventEx` event.

**Post-conditions (Ok)**:
- Target thread resumes execution.

**Invariants**:
```rust
// INVARIANT: ContinueDebugEvent must be called exactly once per WaitForDebugEventEx event.
// VIOLATION: Calling twice → undefined behavior (likely hang or access violation in kernel).
// VIOLATION: Not calling → target process stays suspended; debug loop blocks on next Wait.
```

---

## 6. `SetThreadContext` (decrement RIP after temp BP hit)

**Purpose**: Set RIP back to the pre-INT3 address after the CPU has advanced past it.

| Attribute | Value |
|-----------|-------|
| Signature | `SetThreadContext(hThread: HANDLE, lpContext: *const CONTEXT) -> BOOL` |

**Pre-conditions**:
1. Called inside the `EXCEPTION_BREAKPOINT` handler, before `ContinueDebugEvent`.
2. `CONTEXT.Rip` has been decremented by 1 (to undo the INT3 advance).
3. `CONTEXT.ContextFlags = CONTEXT_ALL_AMD64`.
4. Original byte at `Rip` has already been restored via `WriteProcessMemory`.

**Post-conditions (Ok)**:
- Thread will resume at the original instruction at `Rip - 1 + 1` = `Rip` (the target address).

**Post-conditions (Err)**:
- Context unchanged; do NOT call `ContinueDebugEvent` — emit `EngineEvent::Error` and halt the session.

**Invariants**:
```rust
// INVARIANT: Always restore the original byte BEFORE calling SetThreadContext(Rip-1).
// VIOLATION: If original byte is still 0xCC when execution resumes at Rip-1, the CPU
//            immediately hits INT3 again → infinite breakpoint loop.
//            See issue: TODO(rde#step-over-contract)
```

---

## Composite Protocol: Step-Over (Canonical Order)

```
1. GetThreadContext(tid)                    → ctx.Rip, ctx.Rsp
2. ReadProcessMemory(Rip, 16 bytes)         → insn_bytes
3. capstone::disasm_count(insn_bytes)       → L (instruction length)
4. ReadProcessMemory(Rip+L, 1 byte)         → original_byte  ← SAVE THIS
5. WriteProcessMemory(Rip+L, [0xCC])        → plants temp BP
6. FlushInstructionCache(Rip+L, 1)
7. engine.stepping_over_breakpoint = Some(Rip+L)
8. engine.stepping_over_saved_byte = Some(original_byte)
9. DebugLoopCommand::Continue
   → ContinueDebugEvent(DBG_CONTINUE)
   → WaitForDebugEventEx(...)

On EXCEPTION_BREAKPOINT where ExceptionAddress == Rip+L:
10. WriteProcessMemory(Rip+L, [original_byte])   ← RESTORE
11. FlushInstructionCache(Rip+L, 1)
12. GetThreadContext(tid)                          → ctx (to get new Rip = Rip+L+1, need to decrement)
13. ctx.Rip -= 1                                   ← DECREMENT
14. SetThreadContext(tid, ctx)
15. engine.stepping_over_breakpoint = None
16. engine.stepping_over_saved_byte = None
17. emit EngineEvent::StepCompleted { thread_id, address: Rip+L }
    (DO NOT emit BreakpointHit — this is internal)
```

## Composite Protocol: Step-Out (Canonical Order)

```
1. GetThreadContext(tid)                    → ctx.Rsp
2. ReadProcessMemory(ctx.Rsp, 8 bytes)      → return_addr (u64, little-endian)
3. ReadProcessMemory(return_addr, 1 byte)   → original_byte  ← SAVE THIS
4. WriteProcessMemory(return_addr, [0xCC])  → plants temp BP
5. FlushInstructionCache(return_addr, 1)
6. engine.stepping_out_breakpoint = Some(return_addr)
7. engine.stepping_out_saved_byte = Some(original_byte)
8. DebugLoopCommand::Continue
   → ContinueDebugEvent(DBG_CONTINUE)
   → WaitForDebugEventEx(...)

On EXCEPTION_BREAKPOINT where ExceptionAddress == return_addr:
9.  WriteProcessMemory(return_addr, [original_byte])
10. FlushInstructionCache(return_addr, 1)
11. GetThreadContext(tid)                          → ctx (new Rip = return_addr + 1)
12. ctx.Rip -= 1
13. SetThreadContext(tid, ctx)
14. engine.stepping_out_breakpoint = None
15. engine.stepping_out_saved_byte = None
16. emit EngineEvent::StepCompleted { thread_id, address: return_addr }
```
