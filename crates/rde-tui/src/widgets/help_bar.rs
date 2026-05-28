//! Help bar widget showing context-sensitive keybindings.

use crate::panes::PaneType;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub struct HelpBar;

impl Default for HelpBar {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpBar {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused_pane: PaneType) {
        let help_text = match focused_pane {
            PaneType::Repl => "Enter: send | ↑/↓: history | Tab: next pane | Ctrl+Q: quit",
            _ => "Tab: next pane | Shift+Tab: prev | F5: continue | F9: breakpoint | F10: step over | F11: step into | Ctrl+Q: quit",
        };

        let line = Line::from(vec![
            Span::styled("Help: ", Style::default().fg(Color::DarkGray)),
            Span::raw(help_text),
        ]);

        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}
