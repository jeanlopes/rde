# rde-repl

Read-Eval-Print Loop for interactive debugging.

---

## Overview

The REPL provides an interactive command-line interface to control the debugger and inspect the target process. It communicates with the engine via async channels.

## Running the REPL

The REPL starts automatically when you launch `rde-cli` without `--tui`:

```bash
rde-cli my_app.exe
# REPL prompt appears: rde>
```

## Commands

### Execution Control

| Command | Alias | Engine Command |
|---------|-------|---------------|
| `continue` | `c` | `EngineCommand::Continue` |
| `step` | `s` | `EngineCommand::StepInto` |
| `quit` | `q`, `exit` | `EngineCommand::Quit` |

### Inspection

| Command | Engine Command | Notes |
|---------|---------------|-------|
| `print <expr>` | `EngineCommand::Print` | Pretty-prints if expression is an address |
| `vars` | `EngineCommand::Print { expression: "*" }` | Alias for print all |
| `tasks` | Orchestrator call | Uses `rde_orchestrator::list_tokio_tasks` |
| `regs` | `EngineCommand::ReadRegisters` | |
| `x <addr> [size]` | `EngineCommand::ReadMemory` | Hex dump |
| `bt` | `EngineCommand::Backtrace` | |
| `threads` | `EngineCommand::ListThreads` | |
| `modules` | `EngineCommand::ListModules` | |

### Breakpoints

| Command | Engine Command |
|---------|---------------|
| `break <addr>` | `EngineCommand::SetBreakpoint { address: Some(addr), symbol: None }` |
| `break <symbol>` | `EngineCommand::SetBreakpoint { address: None, symbol: Some(sym) }` |
| `delbreak <id>` | `EngineCommand::DeleteBreakpoint { id }` |

### Configuration

| Command | Effect |
|---------|--------|
| `set auto-disassemble on\|off` | Toggle auto-disassembly on breakpoint hit |
| `set disassembly-count <n>` | Number of instructions to disassemble |
| `set pretty-print on\|off` | Toggle pretty printing (default: on) |
| `set print-limit <n>` | Max collection elements (default: 100) |
| `set print-depth <n>` | Max recursion depth (default: 5) |

### Print Flags

```bash
print [--raw] [--limit N] [--depth N] <expression>
```

| Flag | Description |
|------|-------------|
| `--raw` | Show raw memory instead of pretty-printed value |
| `--limit N` | Override element limit for this command |
| `--depth N` | Override recursion depth for this command |

## Event Handling

The REPL listens to `EngineEvent`s and formats them:

| Event | Display |
|-------|---------|
| `ProcessLaunched { pid }` | `Processo iniciado: PID {pid}` |
| `BreakpointHit { .. }` | `[Breakpoint {id}] Hit em 0x{addr:x} — Thread {tid}` |
| `PrettyValue { value }` | Formatted pretty value |
| `TaskList { tasks }` | Table with ID, State, Function, Thread |
| `MemoryBytes { address, bytes }` | Pretty-printed if `pretty_print` is on |
| `Output { message }` | `{message}` |

## Pretty-Print Integration

When `EngineEvent::MemoryBytes` is received:

1. If `pretty_print` is enabled, the REPL creates a `MemoryReader` from the bytes
2. Looks up a pretty printer in the built-in registry
3. Formats the result and prints it

```rust
let reader = MemoryBytesReader { bytes };
let registry = rde_pretty_print::built_in_registry();
let printer = registry.lookup("std::vec::Vec").unwrap();
let value = printer.format(&reader, 0, &mut budget).unwrap();
println!("{}", format_pretty_value(&value));
```

## Tokio Task Integration

When `tasks` is entered:

1. If a PID is known (from `ProcessLaunched`), calls `rde_orchestrator::list_tokio_tasks(pid)`
2. Formats the result as a table
3. If no PID, falls back to sending `EngineCommand::ListTasks`

## Configuration State

```rust
pub struct ReplConfig {
    pub auto_disassemble: bool,
    pub disassembly_count: usize,
    pub pretty_print: bool,
    pub print_limit: u32,
    pub print_depth: u32,
}
```

Stored in an `Arc<Mutex<ReplConfig>>` shared between the event listener and input loop.
