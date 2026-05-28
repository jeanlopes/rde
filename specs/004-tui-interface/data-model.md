# Data Model: TUI Interface

**Date**: 2026-05-28
**Feature**: TUI Interface with Multi-Pane Layout

## TUI Application State

```rust
/// Top-level application state owned by the TUI event loop.
pub struct TuiApp {
    /// Current layout configuration (pane sizes and positions).
    pub layout: LayoutConfig,
    /// Which pane currently receives keyboard input.
    pub focused_pane: PaneType,
    /// Aggregated debug session state mirrored from EngineEvents.
    pub session_state: SessionMirror,
    /// Channel sender for EngineCommands.
    pub command_tx: UnboundedSender<EngineCommand>,
    /// Whether the TUI should exit on the next frame.
    pub should_quit: bool,
}
```

## Layout Configuration

```rust
/// Describes how terminal area is divided among panes.
pub struct LayoutConfig {
    /// Primary split: vertical (left/right) or horizontal (top/bottom).
    pub primary_split: SplitDirection,
    /// Percentage of terminal width/height allocated to the primary group.
    pub primary_ratio: u16, // 0-100
    /// Panes in the primary group (e.g., Source/ASM + REPL).
    pub primary_panes: Vec<PaneType>,
    /// Panes in the secondary group (e.g., Registers + Stack + Breakpoints).
    pub secondary_panes: Vec<PaneType>,
}

pub enum SplitDirection {
    Vertical,
    Horizontal,
}
```

**Default layout** (80×24 minimum):
```
┌────────────────────┬─────────────┐
│ Source / Assembly  │ Registers   │
│                    ├─────────────┤
│                    │ Stack       │
│                    ├─────────────┤
│                    │ Breakpoints │
├────────────────────┴─────────────┤
│ REPL                             │
└──────────────────────────────────┘
```

## Pane Types

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneType {
    SourceAsm,
    Registers,
    Stack,
    Breakpoints,
    Repl,
}
```

Each pane implements a common trait:

```rust
pub trait Pane {
    /// Render the pane into the given terminal area.
    fn render(&self, frame: &mut Frame, area: Rect, state: &SessionMirror, focus: bool);
    /// Handle a keyboard event when this pane is focused.
    fn handle_key(&mut self, key: KeyEvent, ctx: &mut PaneContext);
    /// Returns the pane type identifier.
    fn pane_type(&self) -> PaneType;
}
```

## Session Mirror

```rust
/// Cached view of the debug session state, updated incrementally by EngineEvents.
pub struct SessionMirror {
    pub state: SessionState, // Running, Paused, Exited
    pub selected_thread: Option<u32>,
    pub threads: Vec<ThreadInfo>,
    pub modules: Vec<ModuleInfo>,
    pub breakpoints: Vec<Breakpoint>,
    pub registers: Option<RegisterContext>,
    pub disassembly: Vec<DisassemblyLine>,
    pub stack_trace: Vec<StackFrame>,
    pub current_address: Option<u64>,
    pub repl_history: Vec<(String, String)>, // (input, output)
    pub status_message: Option<String>,
}
```

## TUI Events

```rust
/// Unified event type for the TUI event loop.
pub enum TuiEvent {
    /// Input from the terminal (crossterm).
    Input(KeyEvent),
    /// Terminal resized.
    Resize(u16, u16),
    /// Event from the debug engine.
    Engine(EngineEvent),
    /// Render tick (driven by a tokio interval).
    Tick,
}
```

## Validation Rules

- `LayoutConfig.primary_ratio` MUST be in the range 30–70 to prevent any pane from becoming unusable.
- `TuiApp.focused_pane` MUST always be a pane that exists in the current layout.
- `SessionMirror` MUST be updated only from the `EngineEvent` stream to maintain consistency with the engine.
