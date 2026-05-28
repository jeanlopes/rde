# Research: TUI Multi-Pane Layout

**Date**: 2026-05-28
**Feature**: TUI Interface with Multi-Pane Layout

## Decision: New Crate `rde-tui`

**Rationale**: The Constitution (Principle II: Crate-First Modularity) mandates every discrete capability be a standalone crate. The TUI is a distinct capability from the REPL, engine, and backend. A dedicated crate prevents monolithic binaries and allows future embedding (e.g., an IDE plugin could reuse `rde-core` without pulling in terminal UI code).

**Alternatives considered**:
- Inline in `rde-cli`: Rejected — violates Crate-First; makes testing harder.
- Inline in `rde-repl`: Rejected — REPL and TUI have different I/O models (line-oriented stdin vs. full-screen raw terminal).

## Decision: `ratatui` + `crossterm`

**Rationale**: `ratatui` is the de-facto standard Rust TUI library. It provides:
- `Layout` system with constraints (percentage, length, min/max) perfect for multi-pane splitting.
- Built-in widgets (`Block`, `Paragraph`, `Table`, `List`) that map directly to our pane types.
- `crossterm` backend for cross-platform terminal handling (Windows 10/11 natively supported).
- Active ecosystem and documentation.

The Constitution already lists `ratatui` under Technology Stack & Architecture Constraints as the intended TUI library (post-MVP).

**Alternatives considered**:
- `tui-rs` (predecessor of ratatui): Rejected — unmaintained; ratatui is the direct successor.
- Custom VT100 escape sequences: Rejected — far too complex; violates Principle VII (Hackable by Design).
- `egui`: Rejected — explicitly forbidden by Constitution (gui/remote-debugging protocols out of MVP scope).

## Decision: Structured `EngineEvent` Variants

**Rationale**: The current engine emits `EngineEvent::Output { message: String }` for almost all query results. A TUI pane needs structured data to render tables, lists, and highlighted lines. Instead of parsing pre-formatted strings (fragile and locale-dependent), we add new typed event variants:

- `EngineEvent::Registers { ctx: RegisterContext }`
- `EngineEvent::Disassembly { lines: Vec<DisassemblyLine> }`
- `EngineEvent::BreakpointList { list: Vec<Breakpoint> }`
- `EngineEvent::StackTrace { frames: Vec<StackFrame> }`

These variants are additive — the text REPL can ignore them or later consume them for richer formatting.

**Alternatives considered**:
- Request/response channels (`tokio::sync::oneshot`): Rejected — heavier refactoring; the existing fire-and-forget model is sufficient if the engine proactively emits structured events after state changes.
- Parse `Output` strings in TUI: Rejected — brittle; breaks on any formatting change.

## Decision: TUI Mode Replaces REPL Mode

**Rationale**: A full-screen TUI application takes over the terminal via raw mode. Running the text REPL simultaneously on the same terminal is impossible without complex terminal multiplexing. The simplest and most reliable model is:
- `rde-cli --tui` launches the TUI.
- `rde-cli` (no flag) launches the existing text REPL.
- The REPL pane inside the TUI is not `rde-repl` reused; it is a native ratatui widget that captures keystrokes and sends `EngineCommand` through the same channel.

**Alternatives considered**:
- Embed `rde-repl` as a subprocess inside a TUI pane: Rejected — adds process orchestration complexity; no clear benefit.
- Run TUI and REPL side-by-side in separate windows: Rejected — out of MVP scope; requires inter-process communication.

## Decision: Stack Walking Implementation in `rde-symbols`

**Rationale**: The TUI spec requires a call stack pane. `DbgHelpSymbolEngine::walk_stack()` is currently stubbed. The implementation uses `StackWalk64` from `DbgHelp.dll`, which is already dynamically loaded. This is a pure `rde-symbols` change with no TUI coupling.

**Trade-off**: This is technically not a TUI feature, but it is a hard dependency for the Stack pane to be functional. It is included in this plan as a prerequisite task.
