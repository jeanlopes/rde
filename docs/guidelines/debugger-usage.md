# Debugger Usage Guide

Complete guide to using the Rust Debugger Engine (RDE) for Windows-native debugging.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Launching a Debug Session](#launching-a-debug-session)
3. [REPL Commands](#repl-commands)
4. [Pretty Printing](#pretty-printing)
5. [Inspecting Tokio Tasks](#inspecting-tokio-tasks)
6. [Cargo Integration](#cargo-integration)
7. [Breakpoints](#breakpoints)
8. [Memory Inspection](#memory-inspection)
9. [Thread Management](#thread-management)
10. [Tips & Tricks](#tips--tricks)

---

## Getting Started

### Prerequisites

- Windows 10/11 x86-64
- Rust toolchain (stable, MSRV 1.78)
- Cargo installed and in PATH

### Building

```bash
cargo build --release
```

The binary is produced at `target/release/rde-cli.exe`.

---

## Launching a Debug Session

### Launch a standalone executable

```bash
rde-cli path/to/debuggee.exe arg1 arg2
```

### Launch from a Cargo project

```bash
cd my-rust-project
rde-cli cargo debug
```

This automatically:
1. Reads `Cargo.toml`
2. Resolves the default binary target
3. Runs `cargo build` if the artifact is stale
4. Launches the debugger with the compiled binary

### Cargo options

```bash
rde-cli cargo debug --package my-crate --bin my-bin --release --features tokio/full
```

| Option | Description |
|--------|-------------|
| `--package <name>` | Package to build (for workspaces) |
| `--bin <name>` | Binary target name |
| `--profile <name>` | Build profile (dev, release, custom) |
| `--features <list>` | Comma-separated features |

### Attach to a running process

```bash
rde-cli
# In REPL:
attach 1234
```

---

## REPL Commands

Once in the REPL (`rde>` prompt), the following commands are available:

### Execution Control

| Command | Alias | Description |
|---------|-------|-------------|
| `continue` | `c` | Resume execution |
| `step` | `s` | Single step (step into) |
| `quit` | `q`, `exit` | Exit debugger |

### Inspection

| Command | Description |
|---------|-------------|
| `print <expr>` | Pretty-print a variable or expression |
| `vars` | List local variables (alias for `print *`) |
| `tasks` | List Tokio async tasks |
| `regs` | Display CPU registers |
| `x <addr> [size]` | Examine memory at address |
| `bt` | Backtrace |
| `threads` | List threads |
| `modules` | List loaded modules |

### Breakpoints

| Command | Description |
|---------|-------------|
| `break <addr>` | Set breakpoint at hex address |
| `break <symbol>` | Set breakpoint at symbol name |
| `delbreak <id>` | Delete breakpoint by ID |

### Configuration

| Command | Description |
|---------|-------------|
| `set auto-disassemble on\|off` | Auto-show disassembly on stop |
| `set disassembly-count <n>` | Lines to disassemble |
| `set pretty-print on\|off` | Enable/disable pretty printing |
| `set print-limit <n>` | Max elements in collections |
| `set print-depth <n>` | Max recursion depth |

---

## Pretty Printing

### Automatic Pretty Printing

When stopped at a breakpoint, RDE automatically pretty-prints standard Rust types:

```
rde> print my_vec
[10, 20, 30]

rde> print my_option
Some(42)

rde> print my_string
"hello, world"

rde> print my_hashmap
{1: "one", 2: "two"}
```

### Supported Types

| Type | Output Example |
|------|---------------|
| `Option<T>` | `Some(value)` / `None` |
| `Vec<T>` | `[1, 2, 3, ...]` |
| `String` | `"hello"` |
| `HashMap<K,V>` | `{1: "a", 2: "b"}` (summary in MVP) |
| `Result<T,E>` | `Ok(value)` / `Err(error)` |

### Print Flags

```bash
# Bypass pretty printing (raw memory)
rde> print --raw my_vec

# Increase element limit
rde> print --limit 500 my_vec

# Increase recursion depth
rde> print --depth 10 my_nested_struct
```

### Persistent Configuration

```bash
rde> set pretty-print on
rde> set print-limit 100
rde> set print-depth 5
```

---

## Inspecting Tokio Tasks

When debugging an async application using Tokio:

```bash
rde> tasks
ID    State      Function                Thread
----  ---------  ----------------------  ------
1     Running    process_requests        12345
2     Sleeping   heartbeat_timer         12346
3     Idle       background_worker       12347
```

If the debuggee does not use Tokio:

```
rde> tasks
No Tokio runtime detected.
```

---

## Cargo Integration

### From CLI

```bash
# Default binary target
rde-cli cargo debug

# Specific package and target
rde-cli cargo debug --package server --bin api-server

# Release build with features
rde-cli cargo debug --release --features "tokio/full,tracing"
```

### Staleness Detection

RDE compares the modification time of the compiled artifact against the newest source file in `src/`. If the artifact is older, `cargo build` is triggered automatically.

### Error Handling

If `cargo build` fails, the error output is forwarded to the terminal and the debug session is not started.

---

## Breakpoints

### Set by Address

```bash
rde> break 0x140001000
Breakpoint 1 definido em 0x140001000
```

### Set by Symbol

```bash
rde> break main
Breakpoint 2 definido em main
```

### List Breakpoints

Breakpoint hits are reported automatically:

```
[Breakpoint 1] Hit em 0x140001000 — Thread 1234
```

### Delete Breakpoint

```bash
rde> delbreak 1
Breakpoint 1 removido.
```

---

## Memory Inspection

### Hex Dump

```bash
rde> x 0x7ff123456000 32
0x00007FF123456000: 48 89 5C 24 08 48 89 74 24 10 57 48 83 EC 20 48  | H..\$.H.t$.WH.. H |
0x00007FF123456010: 8B F9 48 8B F2 48 8B D9 E8 00 00 00 00 48 89 5C  | ..H..H........H.\ |
```

### Pretty-Print from Address

```bash
rde> print 0x7ff123456000
Pretty-print @ 0x7ff123456000:
[10, 20, 30]
```

---

## Thread Management

```bash
rde> threads
 ID       Estado      Selecionada
 1234     Suspended    *
 5678     Running

rde> thread 5678
Thread 5678 selecionada.
```

---

## Tips & Tricks

1. **Auto-disassemble on stop**: `set auto-disassemble on`
2. **Large Vec inspection**: Use `print --limit 500` to see more elements
3. **Nested structures**: Increase depth with `set print-depth 10`
4. **Cargo workspace**: Always specify `--package` in workspaces
5. **Quick recompile**: If you change source code, just `cargo debug` again — RDE detects staleness
