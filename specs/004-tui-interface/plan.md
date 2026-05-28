# Implementation Plan: TUI Interface with Multi-Pane Layout

**Branch**: `004-debugger-tui` | **Date**: 2026-05-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-tui-interface/spec.md`

## Summary

Implement a Terminal User Interface (TUI) with a multi-pane layout that allows the user to simultaneously view source/disassembly, registers, call stack, breakpoints, and an integrated REPL — all within a single terminal window. The TUI will be built as a new workspace crate (`rde-tui`) using `ratatui` and `crossterm`, consuming structured events from the existing `DebugEngine` via the async channel infrastructure already in place.

## Technical Context

**Language/Version**: Rust (stable toolchain; MSRV declared in workspace `Cargo.toml`)

**Primary Dependencies**: `ratatui` (TUI widgets and layout), `crossterm` (terminal input and resize events), `tokio` (async runtime — already in workspace)

**Storage**: N/A (stateless UI; all state lives in `DebugEngine` and is streamed via channels)

**Testing**: `cargo test`, `insta` for UI snapshot tests, manual terminal interaction tests

**Target Platform**: Windows 10/11 (x86-64), terminal with Unicode and ANSI color support

**Project Type**: CLI/TUI application (binary crate in workspace)

**Performance Goals**: UI render loop at 30 FPS; engine-event-to-pane-update latency < 500 ms (per spec SC-004)

**Constraints**: Minimum terminal size 80×24; keyboard-only navigation; must not block the Win32 debug loop thread

**Scale/Scope**: Single-user, single-session local debugger UI; no remote or multi-session support

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Engine-First | ✅ PASS | TUI is a pure consumer of the engine; no Win32 API usage in UI crate |
| II. Crate-First | ✅ PASS | New `rde-tui` crate with single responsibility; independently compilable |
| III. REPL-First | ✅ PASS | TUI sends `EngineCommand` via existing `tokio::mpsc` channels; debug loop untouched |
| IV. Windows-Native | ✅ PASS | No multi-platform abstractions introduced in core or backend crates |
| V. Reference-Driven | N/A | TUI layer does not interact with Win32 debug APIs |
| VI. Rust Safety | ✅ PASS | Pure Rust dependencies (`ratatui`, `crossterm`); no new `unsafe` expected |
| VII. Hackable by Design | ✅ PASS | Single TUI crate; each pane is an isolated widget; no plugin system |
| VIII. Contract-First | ✅ PASS | TUI-Engine contract documented in `contracts/tui-engine-contract.md` |

**Re-check after Phase 1**: All gates still pass. No violations introduced in design.

## API Contracts & Invariants

This feature does not touch `crates/rde-win32` or the debug loop directly. The contract boundary is between `rde-tui` and `rde-core::DebugEngine`.

| API / Boundary | Pre-conditions | Post-conditions (Ok) | Post-conditions (Err) | Invariant |
|----------------|---------------|----------------------|-----------------------|-----------|
| `EngineCommand` send | `command_tx` is alive; engine task is running | Command is queued for engine processing | Channel closed error; TUI exits gracefully | TUI MUST NOT block on send |
| `EngineEvent` receive | `event_rx` is subscribed | Event is dispatched to panes | Channel closed; TUI shows "Session ended" | TUI MUST handle all variants or have a default catch-all |
| `crossterm::event::read` | Terminal is in raw mode | Key/resize event is emitted | Terminal I/O error; TUI restores terminal and exits | Raw mode MUST be restored on panic or exit |
| Pane focus change | At least one pane exists | Focus indicator updated | N/A | Exactly one pane has focus at all times |

**Type-system distinctions:**
- [x] This feature does not introduce system-vs-user event ambiguity. It only consumes `EngineEvent`, which already distinguishes `BreakpointHit` (user) from `Exception` (system/unexpected).
- [x] No magic values are introduced in the TUI layer. Pane identifiers use an enum (`PaneType`).

**Golden path:**
- [ ] After implementation, expected event sequence: `TuiApp starts` → `render initial layout` → `user presses 'c'` in REPL pane → `EngineCommand::Continue` sent → `EngineEvent::ProcessExited` received → `Registers` pane clears, `Source/ASM` pane shows final RIP, status bar updates.
- [ ] Golden path snapshot: `test_data/golden_paths/tui-session.txt`

## Project Structure

### Documentation (this feature)

```text
specs/004-tui-interface/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── tui-engine-contract.md
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
  rde-tui/              # NEW: TUI application crate
    src/
      app.rs            # TuiApp: event loop, state, focus management
      panes/
        mod.rs          # Pane trait and registry
        source_asm.rs   # Source / disassembly pane
        registers.rs    # Register pane
        stack.rs        # Call stack pane
        breakpoints.rs  # Breakpoints pane
        repl.rs         # Embedded REPL pane
      widgets/
        status_bar.rs   # Bottom status line
        help_bar.rs     # Context-sensitive key hints
      lib.rs            # Crate root: TuiApp builder, public run() entry
    Cargo.toml
    tests/
      pane_tests.rs     # Widget unit tests
      snapshots/        # insta snapshots for UI rendering

  rde-core/             # MODIFIED: add structured EngineEvent variants
    src/
      events.rs         # Add Registers, Disassembly, BreakpointList, StackTrace variants

  rde-symbols/          # MODIFIED: implement walk_stack()
    src/
      dbghelp.rs        # Implement StackWalk64 wrapper

  rde-cli/              # MODIFIED: add --tui flag
    src/
      main.rs           # Route to rde_tui::run() when --tui is passed
```

**Structure Decision**: The TUI is a new crate (`rde-tui`) per Principle II (Crate-First). This keeps the TUI decoupled from the text REPL (`rde-repl`) and allows independent evolution. The `rde-cli` binary acts as a thin router choosing between REPL and TUI mode. Minor changes to `rde-core` and `rde-symbols` are required to expose structured data instead of pre-formatted strings.

## Complexity Tracking

> No Constitution Check violations. This section is intentionally left blank.
