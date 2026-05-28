//! Stack / Call stack pane.

use crate::panes::{Pane, PaneContext, PaneType};
use crate::session_mirror::SessionMirror;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct StackPane;

impl Default for StackPane {
    fn default() -> Self {
        Self::new()
    }
}

impl StackPane {
    pub fn new() -> Self {
        Self
    }
}

impl Pane for StackPane {
    fn render(&self, frame: &mut Frame, area: Rect, state: &SessionMirror, focus: bool) {
        let border_style = if focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .title(" Stack ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines: Vec<Line> = if state.stack_trace.is_empty() {
            vec![Line::from("No stack trace available.")]
        } else {
            state
                .stack_trace
                .iter()
                .map(|frame| {
                    let sym_name = frame
                        .symbol
                        .as_ref()
                        .map(|s| s.name.clone())
                        .unwrap_or_else(|| format!("0x{:016x}", frame.return_address));
                    Line::from(vec![
                        Span::styled(
                            format!("#{:<3} ", frame.frame_number),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::raw(sym_name),
                    ])
                })
                .collect()
        };

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &mut PaneContext) {}

    fn pane_type(&self) -> PaneType {
        PaneType::Stack
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
