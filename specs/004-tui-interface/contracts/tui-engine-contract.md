# Contract: TUI ↔ Engine Boundary

**Scope**: Interface between `crates/rde-tui` and `crates/rde-core::DebugEngine`
**Date**: 2026-05-28
**Version**: 1.0.0

## Channel Semantics

### EngineCommand (TUI → Engine)

- **Pre-condition**: The `DebugEngine` task must be running and the `command_tx` channel must be open.
- **Post-condition (Ok)**: The command is enqueued in the engine's command queue. The engine will process it asynchronously.
- **Post-condition (Err)**: If the channel is closed, the TUI MUST treat this as a fatal session error and initiate graceful shutdown.
- **Invariant**: The TUI MUST NOT block the caller thread on `command_tx.send()`. All sends are fire-and-forget.

### EngineEvent (Engine → TUI)

- **Pre-condition**: The TUI must have a live `event_rx` subscription.
- **Post-condition (Ok)**: The event is delivered to the TUI event loop.
- **Post-condition (Err)**: If the channel is closed, the TUI MUST display "Debug session terminated" and stop accepting commands.
- **Invariant**: The TUI MUST handle every `EngineEvent` variant, either with specific logic or a catch-all log/discard path.

## Threading Invariants

- **TUI render thread**: Runs on the main Tokio runtime thread. It MUST NOT perform blocking I/O (including `std::thread::sleep`) outside of `crossterm` event polling, which is non-blocking when configured with `EnableMouseCapture` and raw mode.
- **Debug loop thread**: The dedicated OS thread in `rde-win32` that calls `WaitForDebugEventEx` MUST remain completely independent. The TUI MUST NOT acquire locks or call functions that could block this thread.
- **Channel backpressure**: `tokio::sync::mpsc::unbounded_channel` is used intentionally. The TUI MUST consume events promptly; if the TUI is slower than the event rate, unbounded growth is acceptable for MVP because debug event rates are low (breakpoint hits, not streaming data).

## Structured Event Guarantee

The following new `EngineEvent` variants are guaranteed to carry complete, owned data (no references to engine internals):

| Variant | Payload | Emitted When |
|---------|---------|--------------|
| `Registers { ctx }` | `RegisterContext` | After `EngineCommand::ReadRegisters` or any pause event |
| `Disassembly { lines }` | `Vec<DisassemblyLine>` | After `EngineCommand::Disassemble` |
| `BreakpointList { list }` | `Vec<Breakpoint>` | After any breakpoint mutation (set/delete/hit) |
| `StackTrace { frames }` | `Vec<StackFrame>` | After `EngineCommand::Backtrace` or any pause event |

**Invariant**: These events MUST contain all data necessary to render the corresponding pane without additional queries.

## Coexistence with Text REPL

- The TUI and text REPL are mutually exclusive at runtime.
- The TUI MUST NOT assume it is the sole consumer of `EngineEvent`s. In future designs, multiple subscribers may exist. For MVP, the `rde-cli` binary ensures only one subscriber is active.
