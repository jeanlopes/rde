//! TUI Application — event loop, state, and focus management.

use crate::layout::LayoutConfig;
use crate::panes::{BreakpointsPane, Pane, PaneContext, PaneType, RegistersPane, ReplPane, SourceAsmPane, StackPane};
use crate::session_mirror::SessionMirror;
use crate::widgets::{HelpBar, StatusBar};
use crossterm::event::{Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Clear};
use ratatui::Frame;
use ratatui::Terminal;
use rde_core::{EngineCommand, EngineEvent};
use std::io;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

/// Top-level application state.
pub struct TuiApp {
    pub layout: LayoutConfig,
    pub focused_pane: PaneType,
    pub session: SessionMirror,
    pub command_tx: UnboundedSender<EngineCommand>,
    pub should_quit: bool,
    pub terminal_size: (u16, u16),
    panes: Vec<Box<dyn Pane>>,
    status_bar: StatusBar,
    help_bar: HelpBar,
}

impl TuiApp {
    pub fn new(command_tx: UnboundedSender<EngineCommand>) -> Self {
        let panes: Vec<Box<dyn Pane>> = vec![
            Box::new(SourceAsmPane::new()),
            Box::new(RegistersPane::new()),
            Box::new(StackPane::new()),
            Box::new(BreakpointsPane::new()),
            Box::new(ReplPane::new()),
        ];
        Self {
            layout: LayoutConfig::default(),
            focused_pane: PaneType::SourceAsm,
            session: SessionMirror::new(),
            command_tx,
            should_quit: false,
            terminal_size: (0, 0),
            panes,
            status_bar: StatusBar::new(),
            help_bar: HelpBar::new(),
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        mut event_rx: UnboundedReceiver<EngineEvent>,
    ) -> io::Result<()> {
        info!("TuiApp starting");

        let mut reader = crossterm::event::EventStream::new();
        let mut tick = tokio::time::interval(tokio::time::Duration::from_millis(33));

        while !self.should_quit {
            terminal.draw(|f| self.draw(f))?;

            tokio::select! {
                biased;

                Some(Ok(evt)) = reader.next() => {
                    match evt {
                        CEvent::Key(key) => self.handle_key(key).await,
                        CEvent::Resize(w, h) => {
                            self.terminal_size = (w, h);
                            info!("Terminal resized to {}x{}", w, h);
                        }
                        CEvent::Mouse(_) => {}
                        _ => {}
                    }
                }

                Some(evt) = event_rx.recv() => {
                    self.session.update(&evt);
                    // Force immediate redraw so engine state changes are visible.
                    let _ = terminal.draw(|f| self.draw(f));
                }

                _ = tick.tick() => {}
            }
        }

        info!("TuiApp exiting");
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        if area.width < 80 || area.height < 24 {
            let block = Block::default()
                .title(" Terminal Too Small ")
                .borders(Borders::ALL);
            let text = ratatui::widgets::Paragraph::new(
                "Terminal must be at least 80x24.\nPlease resize your terminal.",
            )
            .block(block);
            frame.render_widget(Clear, area);
            frame.render_widget(text, area);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        let content_area = main_layout[0];
        let status_area = main_layout[1];
        let help_area = main_layout[2];

        let pane_areas = self.layout.calculate(content_area);
        for (pane_type, pane_area) in pane_areas {
            let focus = self.focused_pane == pane_type;
            if let Some(pane) = self.panes.iter().find(|p| p.pane_type() == pane_type) {
                pane.render(frame, pane_area, &self.session, focus);
            }
        }

        self.status_bar.render(
            frame,
            status_area,
            &self.session.state,
            self.session.selected_thread,
            self.session.current_address,
        );
        self.help_bar.render(frame, help_area, self.focused_pane);
    }

    async fn handle_key(&mut self, key: KeyEvent) {
        // Ignore key releases and repeats to avoid double-processing.
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Global shortcuts
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                let _ = self.command_tx.send(EngineCommand::Quit);
                return;
            }
            KeyCode::Tab => {
                self.focus_next();
                return;
            }
            KeyCode::BackTab => {
                self.focus_prev();
                return;
            }
            KeyCode::F(5) => {
                let _ = self.command_tx.send(EngineCommand::Continue);
                return;
            }
            KeyCode::F(9) => {
                if let Some(addr) = self.session.current_address {
                    let _ = self.command_tx.send(EngineCommand::SetBreakpoint {
                        address: Some(addr),
                        symbol: None,
                    });
                }
                return;
            }
            KeyCode::F(10) => {
                let _ = self.command_tx.send(EngineCommand::StepInto);
                return;
            }
            KeyCode::F(11) => {
                let _ = self.command_tx.send(EngineCommand::StepInto);
                return;
            }
            _ => {}
        }

        // Pane-specific handling
        if self.focused_pane == PaneType::Repl {
            if let Some(pane) = self.panes.iter_mut().find(|p| p.pane_type() == PaneType::Repl) {
                if let Some(repl) = pane.as_any_mut().downcast_mut::<ReplPane>() {
                    match key.code {
                        KeyCode::Enter => {
                            let input = repl.input_buffer().to_string();
                            if !input.is_empty() {
                                repl.push_history(input.clone(), String::new());
                                let cmd = parse_repl_input(&input);
                                let _ = self.command_tx.send(cmd);
                                repl.clear_input();
                            }
                        }
                        KeyCode::Up => {
                            if let Some(prev) = repl.history_up() {
                                repl.set_input_buffer(&prev);
                            }
                        }
                        KeyCode::Down => {
                            if let Some(next) = repl.history_down() {
                                repl.set_input_buffer(&next);
                            } else {
                                repl.clear_input();
                            }
                        }
                        _ => {
                            let mut ctx = PaneContext { focused: true };
                            repl.handle_key(key, &mut ctx);
                        }
                    }
                }
            }
        } else {
            let mut ctx = PaneContext { focused: true };
            if let Some(pane) = self.panes.iter_mut().find(|p| p.pane_type() == self.focused_pane) {
                pane.handle_key(key, &mut ctx);
            }
        }
    }

    fn focus_next(&mut self) {
        let panes = self.layout.pane_order();
        if let Some(idx) = panes.iter().position(|&p| p == self.focused_pane) {
            self.focused_pane = panes[(idx + 1) % panes.len()];
        }
    }

    fn focus_prev(&mut self) {
        let panes = self.layout.pane_order();
        if let Some(idx) = panes.iter().position(|&p| p == self.focused_pane) {
            let new_idx = if idx == 0 { panes.len() - 1 } else { idx - 1 };
            self.focused_pane = panes[new_idx];
        }
    }
}

fn parse_repl_input(input: &str) -> EngineCommand {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return EngineCommand::Continue;
    }
    match parts[0].to_lowercase().as_str() {
        "c" | "continue" => EngineCommand::Continue,
        "s" | "step" | "stepinto" => EngineCommand::StepInto,
        "n" | "next" | "stepover" => EngineCommand::StepInto,
        "b" | "break" | "bp" => {
            let addr = parts.get(1).and_then(|a| parse_address(a));
            EngineCommand::SetBreakpoint {
                address: addr,
                symbol: None,
            }
        }
        "d" | "delete" | "del" => {
            let id = parts.get(1).and_then(|a| a.parse().ok()).unwrap_or(0);
            EngineCommand::DeleteBreakpoint { id }
        }
        "r" | "regs" | "registers" => EngineCommand::ReadRegisters { thread_id: None },
        "bt" | "backtrace" => EngineCommand::Backtrace { thread_id: None },
        "m" | "modules" => EngineCommand::ListModules,
        "t" | "threads" => EngineCommand::ListThreads,
        "x" | "read" | "read_mem" => {
            let addr = parts.get(1).and_then(|a| parse_address(a)).unwrap_or(0);
            let size = parts.get(2).and_then(|a| a.parse().ok()).unwrap_or(64);
            EngineCommand::ReadMemory { address: addr, size }
        }
        "disas" | "disassemble" => EngineCommand::Disassemble {
            address: parts.get(1).and_then(|a| parse_address(a)),
            symbol: None,
            thread_id: None,
            count: parts.get(2).and_then(|a| a.parse().ok()),
        },
        "q" | "quit" => EngineCommand::Quit,
        _ => EngineCommand::Continue,
    }
}

fn parse_address(s: &str) -> Option<u64> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse().ok()
    }
}

/// Restore terminal state — called on normal exit AND on panic.
fn restore_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        io::stderr(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    );
}

/// Entry point: run the TUI application.
pub async fn run(
    event_rx: UnboundedReceiver<EngineEvent>,
    command_tx: UnboundedSender<EngineCommand>,
) -> io::Result<()> {
    // Install panic hook so the terminal is never left in raw mode / alternate screen.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        original_hook(info);
    }));

    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = TuiApp::new(command_tx);
    let result = app.run(&mut terminal, event_rx).await;

    restore_terminal();
    terminal.show_cursor()?;

    result
}
