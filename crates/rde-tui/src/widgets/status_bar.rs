//! Status bar widget.

use crate::session_mirror::SessionState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

pub struct StatusBar;

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBar {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &SessionState,
        selected_thread: Option<u32>,
        current_address: Option<u64>,
    ) {
        let state_text = match state {
            SessionState::Running => Span::styled("Running", Style::default().fg(Color::Green)),
            SessionState::Paused => Span::styled("Paused", Style::default().fg(Color::Yellow)),
            SessionState::Exited => Span::styled("Exited", Style::default().fg(Color::Red)),
        };

        let thread_text = selected_thread
            .map(|t| format!("Thread: {t}"))
            .unwrap_or_else(|| "No thread".to_string());

        let addr_text = current_address
            .map(|a| format!("RIP: 0x{a:016x}"))
            .unwrap_or_else(|| "RIP: —".to_string());

        let line = Line::from(vec![
            Span::raw("["),
            state_text,
            Span::raw("]  "),
            Span::raw(thread_text),
            Span::raw("  "),
            Span::raw(addr_text),
        ]);

        let paragraph = Paragraph::new(line).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
}
