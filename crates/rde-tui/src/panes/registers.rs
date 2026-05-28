//! Registers pane.

use crate::panes::{Pane, PaneContext, PaneType};
use crate::session_mirror::SessionMirror;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct RegistersPane;

impl Default for RegistersPane {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistersPane {
    pub fn new() -> Self {
        Self
    }
}

impl Pane for RegistersPane {
    fn render(&self, frame: &mut Frame, area: Rect, state: &SessionMirror, focus: bool) {
        let border_style = if focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .title(" Registers ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines = if let Some(regs) = &state.registers {
            vec![
                Line::from(vec![
                    Span::raw("RAX "),
                    Span::styled(format!("{:016X}", regs.rax), Style::default().fg(Color::Cyan)),
                    Span::raw("  RBX "),
                    Span::styled(format!("{:016X}", regs.rbx), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("RCX "),
                    Span::styled(format!("{:016X}", regs.rcx), Style::default().fg(Color::Cyan)),
                    Span::raw("  RDX "),
                    Span::styled(format!("{:016X}", regs.rdx), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("RSI "),
                    Span::styled(format!("{:016X}", regs.rsi), Style::default().fg(Color::Cyan)),
                    Span::raw("  RDI "),
                    Span::styled(format!("{:016X}", regs.rdi), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("RBP "),
                    Span::styled(format!("{:016X}", regs.rbp), Style::default().fg(Color::Cyan)),
                    Span::raw("  RSP "),
                    Span::styled(format!("{:016X}", regs.rsp), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("RIP "),
                    Span::styled(format!("{:016X}", regs.rip), Style::default().fg(Color::Green)),
                    Span::raw("  RFL "),
                    Span::styled(format!("{:016X}", regs.rflags), Style::default().fg(Color::Magenta)),
                ]),
                Line::from(vec![
                    Span::raw("R8  "),
                    Span::styled(format!("{:016X}", regs.r8), Style::default().fg(Color::Cyan)),
                    Span::raw("  R9  "),
                    Span::styled(format!("{:016X}", regs.r9), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("R10 "),
                    Span::styled(format!("{:016X}", regs.r10), Style::default().fg(Color::Cyan)),
                    Span::raw("  R11 "),
                    Span::styled(format!("{:016X}", regs.r11), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("R12 "),
                    Span::styled(format!("{:016X}", regs.r12), Style::default().fg(Color::Cyan)),
                    Span::raw("  R13 "),
                    Span::styled(format!("{:016X}", regs.r13), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("R14 "),
                    Span::styled(format!("{:016X}", regs.r14), Style::default().fg(Color::Cyan)),
                    Span::raw("  R15 "),
                    Span::styled(format!("{:016X}", regs.r15), Style::default().fg(Color::Cyan)),
                ]),
            ]
        } else {
            vec![Line::from("No register data available.")]
        };

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &mut PaneContext) {}

    fn pane_type(&self) -> PaneType {
        PaneType::Registers
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
