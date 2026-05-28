//! Source / Disassembly pane.

use crate::panes::{Pane, PaneContext, PaneType};
use crate::session_mirror::SessionMirror;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct SourceAsmPane;

impl Default for SourceAsmPane {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceAsmPane {
    pub fn new() -> Self {
        Self
    }
}

impl Pane for SourceAsmPane {
    fn render(&self, frame: &mut Frame, area: Rect, state: &SessionMirror, focus: bool) {
        let border_style = if focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .title(" Source / ASM ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines: Vec<Line> = state
            .disassembly
            .iter()
            .map(|line| {
                let marker = if line.is_current { "=> " } else { "   " };
                let bp_marker = if line.has_breakpoint { "b+ " } else { "   " };
                let style = if line.is_current {
                    Style::default().fg(Color::Green)
                } else if line.has_breakpoint {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                };
                Line::from(vec![
                    Span::raw(format!("{}{}", marker, bp_marker)),
                    Span::styled(
                        format!(
                            "0x{:016x}: {:<20} {} {}",
                            line.address,
                            line.bytes
                                .iter()
                                .take(8)
                                .map(|b| format!("{:02x}", b))
                                .collect::<Vec<_>>()
                                .join(" "),
                            line.mnemonic,
                            line.operands
                        ),
                        style,
                    ),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &mut PaneContext) {}

    fn pane_type(&self) -> PaneType {
        PaneType::SourceAsm
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
