# Implementation Plan: Rust Debugger Engine (RDE)

**Branch**: `001-debugger-engine` | **Date**: 2026-05-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-debugger-engine/spec.md`

## Summary

Implementar o núcleo do Rust Debugger Engine (RDE): uma biblioteca de debugger nativa para
Windows construída do zero sobre a Win32 Debug API. O MVP inclui controle de execução
(launch/attach/continue/step), breakpoints com injeção em runtime, REPL async responsivo,
inspeção de registradores/memória, e stack traces com símbolos Rust. O projeto abandona LLDB
completamente e usa apenas Rust + APIs nativas do Windows.

## Technical Context

**Language/Version**: Rust (stable toolchain, MSRV 1.78+)

**Primary Dependencies**:
- `windows` crate (Microsoft official) — Win32 Debug API, threading, memory
- `tokio` — async runtime, channels (mpsc)
- `capstone` — x86-64 disassembly engine
- `rustc_demangle` — Rust symbol demangling
- `tracing` — structured logging
- `serde` + `serde_json` — serialization for config and snapshots
- `libloading` — dynamic loading of DbgHelp.dll APIs

**Storage**: N/A (in-memory engine; no persistent database in MVP)

**Testing**: `cargo test`, `insta` (snapshot testing), `mockall` (mock backends), integration
tests against `examples/hello_debuggee.rs`

**Target Platform**: Windows 10/11 (x86-64)

**Project Type**: systems library + CLI tool

**Performance Goals**:
- REPL command response < 100ms (99th percentile)
- Runtime breakpoint injection < 50ms
- Session startup to first breakpoint < 5s

**Constraints**:
- Zero runtime dependencies outside Rust crates (`cargo build` must work on a fresh Windows
  machine with only Rust installed)
- `unsafe` permitted ONLY in `crates/rde-win32` with three-condition safety proof
- No LLDB, GDB, Python, PyO3, or external debugger processes
- No Linux/macOS abstractions in core crates

**Scale/Scope**:
- Single-user local debugger (MVP)
- Target: small-to-medium Rust binaries on Windows
- No remote debugging, no JIT expression evaluation, no plugin system

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Compliance | Notes |
|---|---|---|
| I. Engine-First, Not Wrapper-First | ✅ | Engine nativa Win32 do zero; zero LLDB/GDB |
| II. Crate-First Modularity | ✅ | 6 crates no workspace (`rde-core`, `rde-win32`, `rde-symbols`, `rde-breakpoint`, `rde-repl`, `rde-cli`) |
| III. REPL-First Architecture | ✅ | Async channels (`tokio::sync::mpsc`); debug loop isolado em thread dedicada |
| IV. Windows-Native Purity | ✅ | Win32 Debug API exclusiva; DbgHelp para símbolos; PDB único formato MVP |
| V. Reference-Driven Development | ✅ | TitanEngine/x64dbg como referência arquitetural apenas; reimplementação idiomática Rust |
| VI. Rust Safety First | ✅ | `unsafe` apenas em `rde-win32`; Python/LLDB/PyO3 proibidos; Tokio único runtime |
| VII. Hackable by Design | ✅ | MVP sem plugin system, scripting, ou remote debug; doc comments + exemplos obrigatórios |

**Veredicto**: Todos os princípios satisfeitos. Plano pode avançar para Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/001-debugger-engine/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── repl-commands.md
│   └── engine-api.md
└── tasks.md             # Phase 2 output (gerado por /speckit-tasks)
```

### Source Code (repository root)

```text
Cargo.toml                    # Workspace root
rust-toolchain.toml           # MSRV declaration

examples/
  hello_debuggee.rs           # Programa mínimo para testes de integração

crates/
  rde-core/
    src/
      lib.rs                  # Traits do backend, tipos de eventos, state machine
      engine.rs               # Orquestração da sessão de debug
      events.rs               # DebugEvent, EngineEvent, Command enum
      channel.rs              # Wrappers de channels async
    tests/
      engine_tests.rs
    Cargo.toml

  rde-win32/
    src/
      lib.rs                  # WindowsBackend, FFI wrappers
      process.rs              # CreateProcessW, DebugActiveProcess
      debug_loop.rs           # WaitForDebugEventEx, ContinueDebugEvent
      memory.rs               # ReadProcessMemory, WriteProcessMemory
      thread.rs               # Get/SetThreadContext, Suspend/ResumeThread
      module.rs               # Enumeração de DLLs carregadas
    tests/
      win32_tests.rs
    Cargo.toml

  rde-symbols/
    src/
      lib.rs                  # SymbolEngine trait
      dbghelp.rs              # LoadLibrary de DbgHelp.dll, SymInitialize, StackWalk64
      demangler.rs            # rustc_demangle wrapper
    tests/
      symbol_tests.rs
    Cargo.toml

  rde-breakpoint/
    src/
      lib.rs                  # BreakpointManager
      breakpoint.rs           # Estrutura Breakpoint
      hit_handler.rs          # Lógica restore-step-reinstall (INT3)
    tests/
      breakpoint_tests.rs
    Cargo.toml

  rde-repl/
    src/
      lib.rs                  # REPL loop, prompt
      parser.rs               # Parse de comandos textuais
      executor.rs             # Dispatch de comandos para engine
      formatter.rs            # Pretty-print de registradores, memória
    tests/
      parser_tests.rs
    Cargo.toml

  rde-cli/
    src/
      main.rs                 # Entry point, argument parsing (clap)
    Cargo.toml
```

**Structure Decision**: Workspace Cargo com 6 crates separados por responsabilidade. O binário
`rde-cli` consome `rde-core`, que orquestra `rde-win32`, `rde-breakpoint`, `rde-symbols`, e
`rde-repl`. Cada crate é testável isoladamente (`cargo test -p <crate>`).

## Complexity Tracking

> **Constitution Check passou sem violações. Nenhuma justificativa de complexidade necessária.**
