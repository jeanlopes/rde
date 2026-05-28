//! TUI pane widget tests.

use rde_tui::panes::{Pane, RegistersPane, ReplPane, SourceAsmPane};
use rde_tui::session_mirror::SessionMirror;

#[test]
fn test_registers_pane_renders_without_panic() {
    let pane = RegistersPane::new();
    let mut state = SessionMirror::new();
    state.registers = Some(rde_core::RegisterContext {
        rax: 0x1234,
        rbx: 0x5678,
        ..Default::default()
    });

    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            pane.render(f, f.area(), &state, true);
        })
        .unwrap();
}

#[test]
fn test_source_asm_pane_renders_without_panic() {
    let pane = SourceAsmPane::new();
    let mut state = SessionMirror::new();
    state.disassembly = vec![rde_core::DisassemblyLine {
        address: 0x7ff6_0000_0000,
        bytes: vec![0x48, 0x89, 0x5c, 0x24, 0x08],
        mnemonic: "mov".to_string(),
        operands: "qword ptr [rsp+8], rbx".to_string(),
        is_current: true,
        has_breakpoint: false,
        original_bytes: None,
    }];

    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            pane.render(f, f.area(), &state, true);
        })
        .unwrap();
}

#[test]
fn test_repl_pane_command_buffer() {
    let mut pane = ReplPane::new();
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rde_tui::panes::{Pane, PaneContext};

    let mut ctx = PaneContext { focused: true };
    pane.handle_key(
        KeyEvent::from(KeyCode::Char('b')),
        &mut ctx,
    );
    pane.handle_key(
        KeyEvent::from(KeyCode::Char('r')),
        &mut ctx,
    );
    assert_eq!(pane.input_buffer(), "br");
}

#[test]
fn test_repl_pane_history_navigation() {
    let mut pane = ReplPane::new();
    pane.push_history("break 0x1000".to_string(), "OK".to_string());
    pane.push_history("continue".to_string(), "Running".to_string());

    assert_eq!(pane.history_up(), Some("continue".to_string()));
    assert_eq!(pane.history_up(), Some("break 0x1000".to_string()));
    assert_eq!(pane.history_down(), Some("continue".to_string()));
    assert_eq!(pane.history_down(), None);
}

#[test]
fn test_layout_config_recalculates_for_sizes() {
    use rde_tui::layout::LayoutConfig;
    use ratatui::layout::Rect;

    let layout = LayoutConfig::default();

    let small = Rect::new(0, 0, 80, 24);
    let areas_small = layout.calculate(small);
    assert_eq!(areas_small.len(), 5);

    let large = Rect::new(0, 0, 120, 40);
    let areas_large = layout.calculate(large);
    assert_eq!(areas_large.len(), 5);
}
