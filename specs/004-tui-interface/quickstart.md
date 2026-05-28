# Quickstart: TUI Debugger Interface

**Feature**: TUI Interface with Multi-Pane Layout

## Prerequisites

- Windows 10/11 (x86-64)
- Rust toolchain installed (MSRV as declared in root `Cargo.toml`)
- Terminal with Unicode and ANSI color support (Windows Terminal, WezTerm, or Alacritty recommended)

## Build

```bash
cd C:\workspace\rde
cargo build --release
```

## Launch the TUI

```bash
# Launch with TUI mode and auto-start a target
cargo run --bin rde-cli -- --tui --target path\to\debuggee.exe

# Attach to a running process
cargo run --bin rde-cli -- --tui --attach <PID>
```

## Layout

The TUI opens with a multi-pane layout:

```
┌────────────────────┬─────────────┐
│ Source / Assembly  │ Registers   │
│                    ├─────────────┤
│                    │ Stack       │
│                    ├─────────────┤
│                    │ Breakpoints │
├────────────────────┴─────────────┤
│ REPL                             │
└──────────────────────────────────┘
```

## Default Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Cycle focus to next pane |
| `Shift+Tab` | Cycle focus to previous pane |
| `F5` | Continue execution |
| `F10` | Step over |
| `F11` | Step into |
| `F9` | Toggle breakpoint at current line/address |
| `Ctrl+C` | Send interrupt / break into debugger |
| `Ctrl+Q` | Quit TUI and end session |
| `↑/↓` | Scroll within focused pane |
| `Enter` | Submit command (when REPL pane is focused) |

## REPL Commands

When the REPL pane is focused, type commands as you would in the text REPL:

```
> break 0x140001000
> continue
> step
> read_mem 0x7ff6_0000_0000 64
> registers
> backtrace
```

Command output appears in the REPL pane's scrollback area.

## Adjusting Pane Sizes

Use `Ctrl + ←/→` or `Ctrl + ↑/↓` to resize the focused pane (if the terminal supports it).

## Minimum Terminal Size

The TUI requires a terminal of at least **80 columns × 24 rows**. If the terminal is smaller, a warning message is displayed.
