//! REPL pane for the TUI.

use crate::panes::{Pane, PaneContext, PaneType};
use crate::session_mirror::SessionMirror;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct ReplPane {
    input_buffer: String,
    history: Vec<(String, String)>,
    history_cursor: Option<usize>,
    scroll_offset: usize,
}

impl Default for ReplPane {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplPane {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            history: Vec::new(),
            history_cursor: None,
            scroll_offset: 0,
        }
    }

    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    pub fn set_input_buffer(&mut self, text: &str) {
        self.input_buffer.clear();
        self.input_buffer.push_str(text);
    }

    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.history_cursor = None;
    }

    pub fn push_history(&mut self, input: String, output: String) {
        self.history.push((input, output));
        self.scroll_offset = self.history.len().saturating_sub(1);
    }

    pub fn history_up(&mut self) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }
        let idx = match self.history_cursor {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => self.history.len() - 1,
        };
        self.history_cursor = Some(idx);
        Some(self.history[idx].0.clone())
    }

    pub fn history_down(&mut self) -> Option<String> {
        let idx = match self.history_cursor {
            Some(i) if i + 1 < self.history.len() => i + 1,
            _ => {
                self.history_cursor = None;
                return None;
            }
        };
        self.history_cursor = Some(idx);
        Some(self.history[idx].0.clone())
    }
}

impl Pane for ReplPane {
    fn render(&self, frame: &mut Frame, area: Rect, _state: &SessionMirror, focus: bool) {
        let border_style = if focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .title(" REPL ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = self
            .history
            .iter()
            .skip(self.scroll_offset)
            .flat_map(|(input, output)| {
                vec![
                    Line::from(vec![
                        Span::styled("> ", Style::default().fg(Color::Green)),
                        Span::raw(input.clone()),
                    ]),
                    Line::from(Span::raw(output.clone())),
                ]
            })
            .collect();

        lines.push(Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::raw(self.input_buffer.clone()),
            Span::styled("▌", Style::default().fg(Color::Green)),
        ]));

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn handle_key(&mut self, key: KeyEvent, _ctx: &mut PaneContext) {
        match key.code {
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
    }

    fn pane_type(&self) -> PaneType {
        PaneType::Repl
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
