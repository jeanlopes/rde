# Quick Start (Feature 001): RDE Development

**Feature**: 001-debugger-engine
**Date**: 2026-05-28

---

## Running the Example

Build the debuggee:

```powershell
cargo build --example hello_debuggee
```

Start a debug session via the CLI:

```powershell
cargo run --bin rde-cli -- target\debug\examples\hello_debuggee.exe
```

---

## Manual Test Script

```text
rde> break main
rde> continue
[Breakpoint 1] Hit em main (0x...) — Thread ...
rde> regs
rde> step
rde> bt
rde> continue
[Processo encerrado] código: 0
rde> quit
```

---

## Integration Test Verification

Run the integration test suite:

```powershell
cargo test --workspace
cargo test --example hello_debuggee
```

Expected: all tests pass on `windows-latest`.

---

## Architecture Notes for Contributors

- `rde-core` owns the session state machine. Never call Win32 APIs from `rde-core` directly.
- `rde-win32` is the ONLY crate with `unsafe`. Every `unsafe` block requires a safety proof
  comment + GitHub issue link per Constitution VI.
- The debug loop lives in a dedicated `std::thread` inside `rde-win32`, NOT a Tokio task.
- Commands flow: REPL → `EngineCommand` → channel → `DebugEngine` → `DebugBackend`.
- Events flow: `WaitForDebugEventEx` → `DebugEvent` → channel → `DebugEngine` → `EngineEvent` → REPL.
