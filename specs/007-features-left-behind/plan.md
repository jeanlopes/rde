# Implementation Plan: Features Left Behind

**Branch**: `007-features-left-behind` | **Date**: 2026-06-01 | **Spec**: [spec.md](spec.md)

**Input**: [specs/007-features-left-behind/spec.md](spec.md) — 24 user stories derived from 90 `#[ignore]` tests in `tests/big_test_plan.rs`.

---

## Summary

Implement all debugger features blocked by the 90 ignored integration tests. The work falls into five domains: (A) stepping primitives (`next`/`finish`) using Win32 temporary breakpoints, (B) debuggee enhancements to `rust_app_example`, (C) wiring the existing `rde-cargo` crate into the `CargoLaunch` engine command, (D) REPL extensions (`set print-*`, thread selection, script mode), and (E) infrastructure (TUI wiring, Tokio tasks, snapshot testing, performance gates). The final delivery criterion is `cargo test --test big_test_plan` passing with **0 ignored and 0 failed** across all 291 TCs (SC-007).

---

## Technical Context

**Language/Version**: Rust stable (MSRV declared in `workspace.package.rust-version` in root `Cargo.toml`)

**Primary Dependencies**: `windows-rs` 0.58, `tokio` 1.x, `capstone` 0.12, `ratatui` 0.27, `rustc-demangle`, `tracing`, `serde`/`serde_json`

**Storage**: N/A — all debug-session state is in-memory; PDB symbols are read from the file system at launch time.

**Testing**: `cargo test --test big_test_plan` via `RdeSession` integration harness in `tests/common/mod.rs`. Unit tests per crate via `cargo test --package <crate>`.

**Target Platform**: Windows 10/11 x86-64 only.

**Project Type**: CLI binary (`rde-cli`) backed by a workspace of library crates under `crates/`.

**Performance Goals**:
- SC-003: `next` / `finish` respond within 500 ms
- SC-004: `cargo debug` clean build + launch under 30 s
- SC-006: 50-iteration step-inspect loop without memory growth

**Constraints**: Zero-install (`cargo build` is the only required tool). No LLDB, Python, or external debugger processes. `unsafe` only in `crates/rde-win32`.

**Scale/Scope**: ~300 integration test cases; single-user interactive debugger.

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Engine-First | ✅ PASS | `next`/`finish` implemented via Win32 temp-BP technique (read RIP → Capstone decode → write 0xCC → `ContinueDebugEvent` → wait for hit → restore). No LLDB/GDB involvement. |
| II. Crate-First | ✅ PASS | New step primitives go in `rde-win32/src/step.rs`. New command variants in `rde-core/src/lib.rs`. Parser changes in `rde-repl`. No capability siloed in the binary. |
| III. REPL-First | ✅ PASS | `StepOver`/`StepOut` commands transit the existing `mpsc::UnboundedSender<EngineCommand>` channel. The REPL thread remains unblocked during execution. |
| IV. Windows-Native | ✅ PASS | All new Win32 calls (`GetThreadContext`, `ReadProcessMemory`, `SetThreadContext`) are in `rde-win32`. No `#[cfg(unix)]` or cross-platform abstractions introduced. |
| V. Reference-Driven | ✅ PASS | Temp-BP step-over pattern matches x64dbg's `StepOver` implementation (restore-step-reinstall without trap flag). |
| VI. Rust Safety | ✅ PASS | New `unsafe` blocks restricted to `rde-win32/src/step.rs`. Each block requires inline safety proof + GitHub issue reference per constitution §VI. |
| VII. Hackable | ✅ PASS | No plugin systems or scripting engines introduced. Temp-BP logic is ~50 LOC per primitive. |
| VIII. Contract-First | ✅ PASS | New Win32 contracts documented in `specs/007-features-left-behind/contracts/step-primitives.md` and must be ported to `docs/contracts/` before merge. |

**Post-design re-check**: Required after Phase 1. If `BreakpointKind::Temporary` is added, the type-system distinction check (§VIII) must be re-evaluated.

---

## API Contracts & Invariants

*GATE: Required — this feature touches `rde-win32` and the debug loop / engine state machine.*

### Step-Over (`next`) — Temporary Breakpoint Protocol

| API / Boundary | Pre-conditions | Post-conditions (Ok) | Post-conditions (Err) | Invariant |
|----------------|---------------|----------------------|-----------------------|-----------|
| `GetThreadContext` (get RIP) | Thread is suspended (inside `WaitForDebugEventEx` handler) | `rip` field reflects instruction pointer before execution | Context undefined — do not proceed | Call only while process is stopped |
| Capstone `disasm_count(rip, 16)` | Memory at RIP readable; `rip` aligned to valid instruction | Returns `insn[0].bytes.len()` = instruction length L | Returns 0 — fall back to `L = 1` | Always decode before writing temp BP |
| `WriteProcessMemory` (plant 0xCC at RIP+L) | `RIP+L` is a valid writable code page address | Original byte saved; 0xCC written | No write occurred — do not set `stepping_over_breakpoint` | Save original byte before overwriting |
| `ContinueDebugEvent` | Called from the debug-loop thread; IDs match last `WaitForDebugEventEx` result | Target thread resumes | — | Call exactly once per event |
| `WaitForDebugEventEx` → EXCEPTION_BREAKPOINT at RIP+L | Temp BP was planted successfully | `event.Exception.ExceptionRecord.ExceptionAddress == RIP+L` | Other exception — propagate normally | Identify by address match, not by magic ID |
| `WriteProcessMemory` (restore original byte at RIP+L) | BP event received at temp BP address | Original byte restored | Leak: 0xCC remains — log error but continue | Always restore before resuming |
| `SetThreadContext` (set RIP = RIP+L - 1 = original RIP+L) | Restore done; thread suspended | RIP decremented by 1 | Thread state undefined | Decrement exactly once; never decrement on non-temp-BP hit |

**Type-system distinctions:**
- [x] `BreakpointKind::Temporary` MUST be added to distinguish temp BPs (step-over/step-out) from user-defined BPs — prevents accidental `Hit` events being surfaced to the user for internal bookkeeping breakpoints.
- [x] The engine's `stepping_over_breakpoint: Option<u64>` field already provides the address discriminant; the `BreakpointKind` enum must be extended to make this a compile-time distinction.

### Step-Out (`finish`) — Return Address Protocol

| API / Boundary | Pre-conditions | Post-conditions (Ok) | Post-conditions (Err) | Invariant |
|----------------|---------------|----------------------|-----------------------|-----------|
| `GetThreadContext` (get RSP) | Thread suspended inside debug event handler | `rsp` is the current stack pointer | Context undefined | Call only while stopped |
| `ReadProcessMemory` at RSP (read 8 bytes) | RSP is a valid readable stack address | 8 bytes contain the return address as a `u64` | Stack is corrupt or RSP invalid — emit error | Return address is at `[RSP]` on x64 ABI |
| Plant temp BP at return address | Return address is in executable memory | BP set; address saved in engine as `stepping_out_breakpoint` | Address not in any loaded module — emit error | One `finish` → one temp BP; do not stack multiple finish BPs |
| Hit at return address | Temp BP fires | Restore original byte; decrement RIP; stop at caller | — | Remove temp BP atomically before emitting `StepCompleted` event |

**Golden path (Step-Over):**
1. User issues `next` while at `insert` BP hit
2. Engine reads RIP, decodes instruction (e.g., `mov rax, [rbp-8]`, L=4)
3. Writes 0xCC at RIP+4, records `stepping_over_breakpoint = Some(RIP+4)`
4. Sends `DebugLoopCommand::Continue` to debug loop
5. Debug loop calls `ContinueDebugEvent`, `WaitForDebugEventEx`
6. EXCEPTION_BREAKPOINT fires at RIP+4
7. Engine matches address → temporary BP → restore byte, decrement RIP, clear `stepping_over_breakpoint`
8. Engine emits `EngineEvent::StepCompleted { thread_id, address: RIP+4 }`
9. REPL prints `rde>` prompt; user may inspect state

**Golden path snapshot**: `test_data/golden_paths/step_over_insert.txt`

---

## Project Structure

### Documentation (this feature)

```text
specs/007-features-left-behind/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── contracts/
│   └── step-primitives.md   ← Phase 1 output (Win32 contracts)
└── tasks.md             ← Phase 2 output (/speckit-tasks)
```

### Source Code Changes

```text
crates/rde-core/src/
  lib.rs                 ← Add EngineCommand::{StepOver, StepOut, SetPrintConfig}
                            Add EngineEvent::StepCompleted
                            Add BreakpointKind::Temporary

  engine.rs              ← Handle StepOver, StepOut, SetPrintConfig commands
                            Add stepping_out_breakpoint: Option<u64> field
                            Guard commands against ProcessRunning state (US4)
                            Wire CargoLaunch to rde-orchestrator::cargo_resolve_and_build
                            Handle BreakpointKind::Temporary in handle_event

crates/rde-win32/src/
  step.rs  (NEW)         ← step_over(handle, tid) and step_out(handle, tid, read_mem_fn)
  lib.rs                 ← pub mod step;

crates/rde-repl/src/
  parser.rs              ← Add next/n → StepOver, finish/f → StepOut
                            Add set print-limit/print-depth/pretty-print → SetPrintConfig
                            Fix attach parsing to emit Attach command
                            Add script-mode detection (stdin non-TTY)
  lib.rs                 ← Thread prompt suppression in non-TTY mode

crates/rde-breakpoint/src/
  lib.rs                 ← Add BreakpointKind::Temporary; expose set_temporary/remove_temporary

crates/rde-orchestrator/src/
  lib.rs                 ← Wire cargo_resolve_and_build into CargoLaunch handler path
                            Add print_config state forwarding

crates/rde-cli/src/
  main.rs                ← Add --tui flag → launch rde-tui
                            Handle cargo debug subcommand → CargoLaunch
                            Non-TTY stdin detection for script mode

crates/rde-tui/src/
  app.rs                 ← Wire real SessionMirror state into pane renders
                            Handle 'q' for clean exit

examples/rust_app_example/src/
  main.rs                ← Add --demo threaded, --demo panic, --demo stress-test
                            Add enough insertions to trigger AVL rotations
  tree.rs  (NEW or mod)  ← Add TreeStats struct, TreeError enum in scope of insert()

tests/
  big_test_plan.rs       ← Remove #[ignore] tags as each US lands (FR-024)
  common/mod.rs          ← No changes needed

test_data/golden_paths/  (NEW DIR)
  step_over_insert.txt   ← Snapshot for step-over golden path
  full_session.txt       ← Snapshot for TC-262 golden path
```

### Implementation Sequence (dependency order)

| Phase | User Stories | Rationale |
|-------|-------------|-----------|
| **A** — Stepping Primitives | US1, US2, US4 | P1; unblocks 28 TCs; foundational engine work |
| **B** — Debuggee Enhancements | US8, US10, US12, US21 | P2; unblocks 13 TCs; no engine dependency |
| **C** — Cargo Debug | US3 | P1; unblocks 7 TCs; `rde-cargo` ready, just needs wiring |
| **D** — REPL Extensions | US9, US13, US18, US19, US22 | P2/P3; unblocks 16 TCs; parallel with B/C |
| **E** — Infrastructure | US5, US6, US7, US11, US14, US15, US16, US17, US20, US23, US24 | P2/P3; unblocks 26 TCs |
| **F** — Clean Run | FR-024 + SC-007 | Remove all `#[ignore]` tags; full suite passes with 0 ignored |

---

## Complexity Tracking

No constitution violations identified. The plan stays within the existing crate architecture and adds no new abstraction layers beyond the minimum needed for the `BreakpointKind::Temporary` type distinction (required by §VIII contract-first).
