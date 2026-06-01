# Research: Features Left Behind

**Phase**: 0 — Research
**Date**: 2026-06-01
**Feature**: [spec.md](spec.md) | [plan.md](plan.md)

---

## Decision 1 — Step-Over Implementation Technique

**Question**: How should `next` (step-over) be implemented without DWARF line-number tables?

**Decision**: Temporary breakpoint at `RIP + instruction_length` (the "next instruction" approach).

**Rationale**:
- RDE does not use DWARF (PDB only, MVP scope per constitution §IV).
- The trap-flag approach (set TF, catch `SINGLE_STEP` exception) steps one *instruction*, not one *source line*. Without DWARF line tables, there is no way to know which instruction corresponds to the next source line — so a single-step loop that checks line numbers is not viable.
- The temp-BP approach places an INT3 at `RIP + L` (where L is the byte length of the current instruction decoded by Capstone). When `ContinueDebugEvent` is called, the process runs until it either hits the temp BP (stepped over the instruction in the current frame) or hits some other event.
- This is the same technique used by x64dbg's `StepOver` when no source info is available.
- For `next` at the source-line level without DWARF, the best approximation is: advance one machine instruction at a time. This matches the test expectations in `big_test_plan.rs` (tests check only that the REPL returns and the address advances, not that it stops at a specific source line).

**Alternatives considered**:
- *Trap-flag single-step loop with line check*: Requires DWARF `DW_AT_decl_line` — not in scope.
- *Hardware breakpoints (DR0–DR3)*: Limited to 4; would exhaust hardware BPs quickly in multi-step sessions. Temp-BP is simpler and unlimited.
- *Thread suspension + RIP scan*: Complex, race-prone; no benefit over temp-BP.

---

## Decision 2 — Step-Out Implementation Technique

**Question**: How should `finish` (step-out) determine the return address?

**Decision**: Read `[RSP]` (the 8-byte return address at the top of the x64 stack) and plant a temp BP there.

**Rationale**:
- On x64 Windows ABI, the return address is always at `[RSP]` when the callee has not yet set up its own frame pointer. Even after `push rbp; mov rbp, rsp`, the return address is at `[RBP+8]`. Since we have `RBP` from `GetThreadContext`, we can also read `[RBP+8]` as a more reliable alternative if the function has a frame pointer.
- Primary method: use `RSP` directly — works even for frameless functions.
- Fallback: if `RSP` produces an address outside all loaded module ranges (detected via `session.modules`), try `[RBP+8]`.

**Alternatives considered**:
- *Stack walking via `StackWalk64`*: Returns the full call chain; we only need the immediate return address. Heavier than necessary.
- *DWARF unwind info*: Out of scope.

---

## Decision 3 — Cargo Debug Integration Architecture

**Question**: Where does `cargo build` invocation live, and how does the binary path flow into `Launch`?

**Decision**: Keep it in `rde-cargo` crate; call `cargo_resolve_and_build()` from the engine's `CargoLaunch` handler via `rde-orchestrator`.

**Rationale**:
- `rde-cargo` already implements `fetch_metadata()`, `resolve_target()`, `is_stale()`, and `run_build()` (confirmed by codebase exploration).
- `rde-orchestrator` already imports `rde-cargo` and has a `cargo_resolve_and_build()` function stub.
- The engine's `CargoLaunch` handler currently emits only an `Output` message. The fix is: call `orchestrator::cargo_resolve_and_build(...)`, get back a `PathBuf`, then invoke `self.backend.launch(&binary_path, &args)` exactly as the normal `Launch` handler does.
- This avoids adding `rde-cargo` as a direct dependency of `rde-core`, keeping the dependency graph clean.

**Alternatives considered**:
- *Call `cargo build` directly in `rde-cli/main.rs`*: Violates Crate-First (capability would be in the binary, not a library crate).
- *Spawn `cargo` via `std::process::Command` in `rde-core`*: `rde-core` should not invoke shell processes; that belongs in `rde-cargo`.

---

## Decision 4 — Multi-Threaded Debuggee Approach

**Question**: Should the multi-threaded debuggee be a new binary, a new demo mode in `rust_app_example`, or the existing `threaded_debuggee.rs` example?

**Decision**: Add `--demo threaded` mode to `rust_app_example/src/main.rs` that spawns 3 worker threads, each inserting values into a shared BST.

**Rationale**:
- The existing `crates/rde-cli/examples/threaded_debuggee.rs` spawns threads but is in the CLI crate's examples directory — not reachable by `tests/big_test_plan.rs` `debuggee_path()` (which points to `rust_app_example`).
- Adding a new demo mode keeps all debuggee functionality in one binary, consistent with the existing `--demo` pattern.
- Worker threads share a `Mutex<BinaryTree>` and each insert 3 values; this creates real thread-creation events without requiring lock-free data structures.

**Alternatives considered**:
- *Separate binary*: Would require `debuggee_path()` to be parameterizable; changes the test harness.
- *Reuse `threaded_debuggee.rs` example*: Path mismatch with existing test helpers.

---

## Decision 5 — AVL Rotation Trigger in Debuggee

**Question**: The current `--demo insert-sequence` does not trigger `rotate_left` or `rotate_right`. How many and which values must be inserted to force a rotation?

**Decision**: Insert values `[1, 2, 3, 4, 5]` in order. This guarantees a right-rotation on the third insert (1→2→3 is right-heavy, triggers left-rotation to balance at node 2) and further rotations on subsequent inserts.

**Rationale**:
- AVL property: balance factor of any node must be in `{-1, 0, 1}`. Inserting 1, 2, 3 in order creates a right-right imbalance → left-rotation at node 1.
- Inserting 4, 5 creates another right-right imbalance → left-rotation at node 3.
- This keeps the demo predictable and short (5 values instead of the current variable-length sequence).
- The existing `--demo insert-sequence` can be updated to use this sequence, or a new `--demo avl-rotations` mode can be added. Recommendation: update the existing `insert-sequence` with additional values `[6, 7, 8, 9, 10]` to ensure both left- and right-rotations occur.

---

## Decision 6 — Pretty Print Configuration Commands

**Question**: How should `set print-limit N`, `set print-depth N`, and `set pretty-print on|off` be dispatched?

**Decision**: New `EngineCommand::SetPrintConfig { limit: Option<usize>, depth: Option<usize>, pretty: Option<bool> }` stored on the engine and forwarded to `rde-orchestrator`'s `pretty_print_value()` calls via a `PrintConfig` struct.

**Rationale**:
- Mirrors the existing `SetDisassemblyConfig` pattern exactly — same shape, same dispatch path, no new architectural concepts.
- `PrintConfig` struct can live in `rde-core` (pure data) and be referenced by `rde-orchestrator`.

---

## Decision 7 — Script / Batch Mode

**Question**: How should stdin piping be detected to suppress interactive prompts?

**Decision**: Use `is_terminal::IsTerminal::is_terminal()` on `std::io::stdin()` in `rde-repl`. If not a terminal, suppress the `rde>` prompt string but still process commands line-by-line.

**Rationale**:
- `is-terminal` is a lightweight crate already used by many Rust CLIs; adds one dependency but is idiomatic.
- Alternative: `atty` crate (deprecated in favor of `is-terminal`).
- This matches how GDB and LLDB handle piped input.

---

## Decision 8 — Temporary Breakpoint Identity

**Question**: How should the engine distinguish a temp BP hit from a user BP hit?

**Decision**: Add `BreakpointKind::Temporary` to the existing enum and store the temp BP address in a separate `stepping_over_breakpoint: Option<u64>` / `stepping_out_breakpoint: Option<u64>` on the engine. When a `BreakpointHit` arrives, check address against these fields first, before checking `breakpoints.find_by_address()`.

**Rationale**:
- `stepping_over_breakpoint` already exists on `DebugEngine` (confirmed in `engine.rs` line 49).
- Adding `stepping_out_breakpoint: Option<u64>` is symmetric.
- The type-system check (§VIII) is satisfied because `BreakpointKind::Temporary` is a distinct match arm — the compiler enforces separate handling paths.
- No temp BPs are ever added to `BreakpointManager` (which is the user-facing list), keeping `listbreaks` output clean.

---

## Summary of Resolved Unknowns

| Unknown | Resolved Decision |
|---------|------------------|
| Step-over technique | Temp BP at RIP+L via Capstone decode |
| Step-out return address | Read `[RSP]` (fallback: `[RBP+8]`) |
| Cargo debug wiring | `CargoLaunch` handler → `orchestrator::cargo_resolve_and_build()` → `Launch` |
| Multi-thread debuggee | New `--demo threaded` mode in `rust_app_example` |
| AVL rotation trigger | Insert `[1..10]` in `--demo insert-sequence` |
| Print config dispatch | New `SetPrintConfig` command mirroring `SetDisassemblyConfig` pattern |
| Script mode detection | `is-terminal` crate on `stdin()` |
| Temp BP identity | Separate `stepping_over/out_breakpoint: Option<u64>` fields + `BreakpointKind::Temporary` |
