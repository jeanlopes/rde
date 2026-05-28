//! Pane trait and pane type definitions.

pub mod breakpoints;
pub mod registers;
pub mod repl;
pub mod source_asm;
pub mod stack;

pub use breakpoints::BreakpointsPane;
pub use registers::RegistersPane;
pub use repl::ReplPane;
pub use source_asm::SourceAsmPane;
pub use stack::StackPane;

use crate::session_mirror::SessionMirror;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Types of panes in the TUI layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneType {
    SourceAsm,
    Registers,
    Stack,
    Breakpoints,
    Repl,
}

/// Context passed to pane handlers.
pub struct PaneContext {
    pub focused: bool,
}

/// Common trait for all TUI panes.
pub trait Pane {
    /// Render the pane into the given terminal area.
    fn render(&self, frame: &mut Frame, area: Rect, state: &SessionMirror, focus: bool);
    /// Handle a keyboard event when this pane is focused.
    fn handle_key(&mut self, key: KeyEvent, ctx: &mut PaneContext);
    /// Returns the pane type identifier.
    fn pane_type(&self) -> PaneType;
    /// Downcast to Any for concrete pane access.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
