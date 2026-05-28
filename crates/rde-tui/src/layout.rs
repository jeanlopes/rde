//! Layout configuration for multi-pane TUI.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Describes how terminal area is divided among panes.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub direction: Direction,
    pub primary_ratio: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            direction: Direction::Horizontal,
            primary_ratio: 65,
        }
    }
}

impl LayoutConfig {
    /// Calculate pane areas for the given terminal size.
    pub fn calculate(&self, area: Rect) -> Vec<(crate::panes::PaneType, Rect)> {
        let main_chunks = Layout::default()
            .direction(self.direction)
            .constraints([
                Constraint::Percentage(self.primary_ratio),
                Constraint::Percentage(100 - self.primary_ratio),
            ])
            .split(area);

        let left = main_chunks[0];
        let right = main_chunks[1];

        // Left column: Source/ASM (top 70%) + REPL (bottom 30%)
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(left);

        // Right column: Registers (top) + Stack (middle) + Breakpoints (bottom)
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(right);

        vec![
            (crate::panes::PaneType::SourceAsm, left_chunks[0]),
            (crate::panes::PaneType::Registers, right_chunks[0]),
            (crate::panes::PaneType::Stack, right_chunks[1]),
            (crate::panes::PaneType::Breakpoints, right_chunks[2]),
            (crate::panes::PaneType::Repl, left_chunks[1]),
        ]
    }

    pub fn pane_order(&self) -> Vec<crate::panes::PaneType> {
        vec![
            crate::panes::PaneType::SourceAsm,
            crate::panes::PaneType::Registers,
            crate::panes::PaneType::Stack,
            crate::panes::PaneType::Breakpoints,
            crate::panes::PaneType::Repl,
        ]
    }
}
