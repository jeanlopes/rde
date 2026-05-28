# Research: Rust Debugger Engine (RDE)

**Date**: 2026-05-28
**Feature**: 001-debugger-engine

---

## Decision: Async Runtime — Tokio vs crossbeam

**Decision**: Use `tokio` as the primary async runtime with `tokio::sync::mpsc` for channels.

**Rationale**:
- Constitution VI mandates Tokio as the sole async runtime.
- `tokio::sync::mpsc` provides bounded/unbounded channels with backpressure, which is critical
  for the debug event loop (producer) and REPL (consumer) separation.
- Tokio's `spawn_blocking` can be used for the `WaitForDebugEventEx` loop if needed, though
  a dedicated std::thread is preferred for the debug loop to avoid tokio scheduler interference.

**Alternatives considered**:
- `crossbeam-channel`: Excellent performance, but using it alongside Tokio creates a mixed-async
  model. Since the constitution requires Tokio-only, we standardize on `tokio::sync::mpsc`.
- `async-channel`: Third-party crate, unnecessary when tokio provides equivalent functionality.

---

## Decision: DbgHelp Integration Strategy

**Decision**: Load `DbgHelp.dll` dynamically via `libloading` at runtime, rather than static
linking or `windows` crate bindings.

**Rationale**:
- `DbgHelp.dll` is a system DLL present on all Windows installations. Dynamic loading avoids
  build-time linking complexity.
- The `windows` crate does not provide first-class bindings for all DbgHelp APIs (e.g.,
  `StackWalk64`, `SymGetLineFromAddr64`).
- Dynamic loading allows graceful degradation: if symbol resolution fails, the debugger still
  functions for memory/register operations.

**Alternatives considered**:
- `pdb` crate (pure Rust PDB parser): Attractive for zero-unsafe, but reimplementing stack
  unwinding and line information lookup from scratch is a massive undertaking. DbgHelp is the
  battle-tested path.
- Static linking against dbghelp.lib: Possible but creates version coupling with Windows SDK.

---

## Decision: Breakpoint Implementation Pattern

**Decision**: Use the INT3 (`0xCC`) opcode injection pattern with restore-step-reinstall.

**Rationale**:
- This is the universal software breakpoint mechanism used by x64dbg, WinDbg, Visual Studio,
  and TitanEngine.
- Pattern: save original byte → write `0xCC` → continue → on `STATUS_BREAKPOINT`, restore byte,
  decrement RIP, set Trap flag → single-step → reinstall `0xCC`.
- Hardware breakpoints (DR0-DR3 registers) are limited to 4 and require more complex context
  management. Software breakpoints are unlimited and simpler for the MVP.

**Alternatives considered**:
- Hardware breakpoints: Limited slots, more context switching, but no memory modification.
  Useful for read/write watchpoints (future feature).
- Page guard breakpoints (`PAGE_GUARD`): Useful for memory watchpoints but overkill for code
  breakpoints and can cause performance degradation.

---

## Decision: Channel Architecture for REPL-First

**Decision**: Two unbounded `tokio::sync::mpsc` channels:
- `command_tx` / `command_rx`: REPL → Engine (commands like SetBreakpoint, Continue)
- `event_tx` / `event_rx`: Engine → REPL (events like BreakpointHit, ProcessExited)

**Rationale**:
- The debug loop thread runs `WaitForDebugEventEx` in a blocking loop. When an event arrives,
  it sends a clone via `event_tx` and immediately calls `ContinueDebugEvent`.
- Between `WaitForDebugEventEx` calls, the loop checks `command_rx.try_recv()` for pending
  commands without blocking.
- This guarantees the REPL thread is never blocked by the target process execution.

**Alternatives considered**:
- Shared state (Arc<Mutex<EngineState>>): Rejected due to risk of deadlocks between REPL and
  debug loop threads. Constitution III explicitly requires message passing.
- `tokio::sync::broadcast` for events: Rejected because only the REPL consumes events; broadcast
  is overkill.

---

## Decision: Testing Strategy for Windows-Specific Code

**Decision**: Three-tier testing:
1. **Unit tests**: Mock `DebugBackend` trait using `mockall` for `rde-core` state machine logic.
2. **Integration tests**: Real `hello_debuggee.exe` process spawned and debugged in CI
   (`windows-latest` GitHub Actions runner).
3. **Snapshot tests**: `insta` for REPL output formatting (registers, memory dump, stack trace).

**Rationale**:
- Windows API calls (`ReadProcessMemory`, etc.) cannot be easily mocked without complex FFI
  interception. Integration tests against a real process are essential.
- `hello_debuggee.rs` is a minimal, deterministic program that exercises all debug events.
- Snapshot tests catch regressions in output formatting without brittle string assertions.

---

## Decision: REPL Command Parser Strategy

**Decision**: Start with a hand-written recursive descent parser (manual tokenization) for the
MVP. Migrate to `winnow` or `chumsky` only if command grammar becomes complex.

**Rationale**:
- MVP commands are simple: `break <addr|sym>`, `continue`, `step`, `regs`, `x <addr> <size>`,
  `bt`, `quit`. A hand-written parser is ~200 lines and has zero dependencies.
- Adding a parser combinator crate is acceptable per the constitution but unnecessary complexity
  for the initial command set.
- Decision documented for future reevaluation when TUI and auto-complete are introduced.

**Alternatives considered**:
- `chumsky`: Excellent error recovery and composability, but pulls in multiple dependencies.
- `winnow`: Zero-copy, fast, but learning curve for team members.
- `clap` (for CLI args only): Already used in `rde-cli` for startup arguments, but unsuitable
  for interactive REPL commands.

---

## Open Questions Resolved

| Question | Resolution |
|---|---|
| How to handle 32-bit WOW64 processes? | Out of MVP scope. RDE targets x86-64 only. WOW64 support may be added later. |
| Should we support attach to system services? | No. UAC elevation is required but not guaranteed. Document limitation. |
| Memory write safety — should we require confirmation? | Yes for writes > 8 bytes or to executable regions. Under 8 bytes to data regions: silent. |
