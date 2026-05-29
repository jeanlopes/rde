# Quickstart: Rust Debug Extensions

**Feature**: Rust Debug Extensions (pretty printers, Tokio tasks, Cargo integration)
**Date**: 2026-05-28

---

## Prerequisites

- Windows 10/11 x86-64
- Rust toolchain (stable) installed
- Cargo installed and available in `PATH`
- RDE built from source (`cargo build --release` in workspace root)

---

## Launching a Cargo Project

1. Navigate to a Cargo project directory:
   ```powershell
   cd C:\my-project
   ```

2. Start RDE with Cargo integration:
   ```powershell
   rde cargo debug
   ```
   RDE will:
   - Read `Cargo.toml` and resolve the default binary target.
   - Check if the artifact in `target/debug/` is stale.
   - Run `cargo build` automatically if needed.
   - Launch the debuggee and drop you into the REPL.

3. For a specific package or target:
   ```powershell
   rde cargo debug --package my-crate --bin my-bin --release --features tokio/full
   ```

---

## Using Pretty Printers

While stopped at a breakpoint, inspect variables:

```
(rde) print my_vec
[10, 20, 30]

(rde) print my_option
Some(42)

(rde) print my_string
"hello, world"

(rde) print my_hashmap
{1: "one", 2: "two"}
```

Pretty printers apply automatically when the variable's type is recognized from PDB symbols. Use `print --raw <var>` to bypass pretty printing and see raw memory layout.

---

## Inspecting Tokio Tasks

While debugging an async application using Tokio:

```
(rde) tasks
ID    State      Function                Thread
----  ---------  ----------------------  ------
1     Running    process_requests        12345
2     Sleeping   heartbeat_timer         12346
3     Idle       background_worker       12347
```

If the debuggee does not use Tokio, the command returns:
```
No Tokio runtime detected in the target process.
```

---

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| `cargo debug` says "Cargo.toml not found" | Not in a Cargo project directory | `cd` to the correct directory |
| `cargo build` fails | Compilation error in project | Fix the project code; RDE will show stderr |
| Variables print as raw bytes | PDB not loaded or type unknown | Ensure debug build with `-g`; check `modules` command |
| `tasks` shows no tasks | Tokio runtime not yet initialized | Break after `#[tokio::main]` starts |
| Large `Vec` is truncated | Default element limit (100) reached | Use `print --limit 500 my_vec` |
