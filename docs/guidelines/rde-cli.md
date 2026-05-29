# rde-cli

Command-line interface for the Rust Debugger Engine.

---

## Usage

```bash
rde-cli [OPTIONS] [TARGET] [TARGET_ARGS...]
rde-cli cargo debug [CARGO_OPTIONS]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `TARGET` | Path to executable to debug |
| `TARGET_ARGS` | Arguments passed to the target |

## Options

| Option | Description |
|--------|-------------|
| `--tui` | Run with TUI interface instead of text REPL |

## Subcommands

### `cargo debug`

Launch a debug session from a Cargo project.

```bash
rde-cli cargo debug [OPTIONS]
```

#### Cargo Options

| Option | Description |
|--------|-------------|
| `--package <NAME>` | Package to build (for workspaces) |
| `--bin <NAME>` | Binary target to build |
| `--profile <NAME>` | Build profile (dev, release, custom) |
| `--features <LIST>` | Comma-separated features |

## Examples

### Launch standalone executable

```bash
rde-cli target/debug/my_app.exe --arg1 --arg2
```

### Launch from Cargo project

```bash
cd my_project
rde-cli cargo debug
```

### Launch with specific package and features

```bash
rde-cli cargo debug --package server --bin api --features "tokio/full"
```

### TUI mode

```bash
rde-cli --tui target/debug/my_app.exe
```

## Architecture

`rde-cli` is the binary entry point. It:

1. Parses arguments with `clap`
2. Creates `WindowsBackend`
3. Creates `DebugEngine`
4. Spawns the engine loop
5. Runs TUI or REPL
6. For `cargo debug`, uses `rde-orchestrator` to resolve/build before launch

## Integration Points

```rust
use rde_core::DebugEngine;
use rde_win32::WindowsBackend;

let backend = WindowsBackend::new();
let (engine, command_tx, event_rx) = DebugEngine::new(backend);

// For cargo debug:
let artifact = rde_orchestrator::cargo_resolve_and_build(
    &manifest_path, package, target, profile, features
).await?;

command_tx.send(EngineCommand::Launch { path: artifact, args: vec![] });
```
