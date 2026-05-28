//! Breakpoints pane.

use crate::panes::{Pane, PaneContext, PaneType};
use crate::session_mirror::SessionMirror;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct BreakpointsPane;

impl Default for BreakpointsPane {
    fn default() -> Self {
        Self::new()
    }
}

impl BreakpointsPane {
    pub fn new() -> Self {
        Self
    }
}

impl Pane for BreakpointsPane {
    fn render(&self, frame: &mut Frame, area: Rect, state: &SessionMirror, focus: bool) {
        let border_style = if focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .title(" Breakpoints ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines: Vec<Line> = if state.breakpoints.is_empty() {
            vec![Line::from("No breakpoints set.")]
        } else {
            state
                .breakpoints
                .iter()
                .map(|bp| {
                    let state_str = match bp.state {
                        rde_core::BreakpointState::Enabled => "E",
                        rde_core::BreakpointState::Disabled => "D",
                        rde_core::BreakpointState::Pending => "P",
                    };
                    Line::from(vec![
                        Span::styled(format!("{:<4}", bp.id), Style::default().fg(Color::Cyan)),
                        Span::raw(" "),
                        Span::styled(state_str, Style::default().fg(Color::Green)),
                        Span::raw(" "),
                        Span::raw(format!("0x{:016x}", bp.address)),
                        Span::raw(format!(" (hits: {})", bp.hit_count)),
                    ])
                })
                .collect()
        };

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &mut PaneContext) {}

    fn pane_type(&self) -> PaneType {
        PaneType::Breakpoints
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
