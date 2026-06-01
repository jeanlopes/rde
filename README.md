# RDE — Rust Debugger Engine

Rust Observability Engine

Debugger nativo Windows escrito em Rust, do zero, sem LLDB.

## O que é

RDE é uma biblioteca de debugger engine e uma CLI REPL para Windows, construída diretamente sobre
as APIs nativas de debugging do Win32 (`WaitForDebugEventEx`, `ReadProcessMemory`,
`WriteProcessMemory`, etc.). Ela existe porque LLDB é um cidadão de segunda classe no Windows:
FFI frágil, UI bloqueante, breakpoints que não podem ser injetados em runtime, e um ecossistema
Python que cria mais problemas do que resolve.

RDE não é um wrapper. É um engine.

## Por que RDE?

| Problema com LLDB | Como o RDE resolve |
|---|---|
| UI congela durante execução | Arquitetura async com message passing — REPL responde sempre |
| Não dá pra adicionar breakpoint em runtime | `SuspendThread` + `WriteProcessMemory` via channel, sem travar |
| PDB problemático no LLDB | `DbgHelp.dll` nativa do Windows, feita pela Microsoft |
| FFI com C++ (lldb-sys) | Zero FFI. Apenas `windows-rs` e syscalls Win32 |
| Código legado monstruoso | Código Rust moderno, modular, hackable |

## Features

- **Launch e attach** de processos Windows via `CreateProcessW`/`DebugActiveProcess`
- **Breakpoints em runtime** — adicione, remova e gerencie breakpoints sem pausar o REPL
- **Single-step** via `SetThreadContext` (flag Trap)
- **Inspeção de registradores** x64 completos (RAX–R15, RIP, RFLAGS, etc.)
- **Leitura/escrita de memória** com `ReadProcessMemory`/`WriteProcessMemory`
- **Stack traces** com resolução de símbolos via `DbgHelp` + demangling Rust
- **Disassembly** com Capstone Engine
- **REPL embutido** com comandos estilo GDB/LLDB (`break`, `continue`, `step`, `regs`, `x`, etc.)
- **Arquitetura 100% Rust** — sem Python, sem LLVM, sem processos externos

## Requisitos

- Windows 10 ou 11 (x86-64)
- Rust (stable toolchain) — [rustup.rs](https://rustup.rs)
- Nada mais. Zero dependências de sistema.

## Stack

| Camada | Tecnologia |
|---|---|
| Win32 API | `windows` (oficial Microsoft) |
| Disassembly | `capstone` |
| Símbolos/PDB | `DbgHelp.dll` |
| Async/Channels | `tokio` |
| TUI (futuro) | `ratatui` |
| Logging | `tracing` |

## Estrutura do Workspace

```
crates/
  rde-core/         — Traits, tipos, state machine do engine, canais de evento
  rde-win32/        — Backend Win32 (CreateProcessW, WaitForDebugEventEx, etc.)
  rde-symbols/      — Integração com DbgHelp, demangling, stack walking
  rde-breakpoint/   — Gerenciamento de breakpoints (set/remove/hit/reinstall)
  rde-repl/         — Parser de comandos, executor, loop REPL
  rde-cli/          — Binário CLI

examples/
  hello_debuggee.rs — Programa mínimo para testes de integração
```

## Quick Start

Veja [docs/quickstart.md](docs/quickstart.md) para a primeira sessão de debug passo a passo.

Build rápido:

```powershell
cargo build --workspace
cargo run --bin rde-cli -- examples\target\debug\hello_debuggee.exe
```

## Arquitetura

```
┌──────────────┐
│ CLI / REPL   │
└──────┬───────┘
       │ commands via channel
       ▼
┌──────────────┐
│ Debug Engine │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Win32 Backend│
│ WaitForDebug │
│ Read/Write   │
└──────────────┘
```

O segredo é a separação de threads:
- **Main thread**: REPL input, renderização
- **Debug Loop thread**: `WaitForDebugEventEx` exclusivo
- **Symbol Worker thread**: resolução de símbolos (pesado, não bloqueia)

## Inspirado em

- [TitanEngine](https://github.com/x64dbg/TitanEngine) — arquitetura de referência
- [x64dbg](https://github.com/x64dbg/x64dbg) — separação engine/UI
- [Writing a Linux Debugger](https://blog.tartanllama.xyz/writing-a-linux-debugger/) — conceitos de debugger (adaptados para Win32)

## Constituição do Projeto

As regras não-negociáveis do projeto estão em `.specify/memory/constitution.md`. Todo PR deve
incluir uma auto-avaliação de conformidade.

## Contribuindo

1. Leia a constituição (`.specify/memory/constitution.md`)
2. Siga o fluxo: `/speckit-specify` → `/speckit-clarify` → `/speckit-plan` → `/speckit-tasks` → `/speckit-implement`
3. Testes primeiro. Red-Green-Refactor.

## Licença

MIT ou Apache-2.0 (a definir)



#	Spec	Fases do Plano	Escopo	User Stories Principais
001	debugger-engine	0–5	MVP Core — Debug loop, breakpoints runtime, REPL, registradores, memória, stack trace, símbolos	Launch/attach, REPL responsivo, inspeção de estado  
002	thread-module-tracking	6	Rastreamento de threads e módulos (DLLs) dinamicamente	Listar/trocar threads, listar módulos carregados  
003	disassembly-view	7	Visualização de código assembly com destaque de breakpoints	Disassembly em endereços específicos, sincronia com RIP  
004	tui-interface	8	Interface terminal gráfica (ratatui) com painéis	Layout multi-pane (source/asm, regs, stack, breakpoints, REPL)  
005	rust-debug-extensions	9	Diferenciais Rust: pretty printers, async tasks, cargo integration	Visualizar Option<T>, Vec<T>, tasks tokio, cargo debug  

# Compilar os binários primeiro
cargo build -p rde-cli -p rust_app_example

# Rodar todos os testes ativos
cargo test --test big_test_plan

# Com output visível
cargo test --test big_test_plan -- --nocapture

# Um TC específico
cargo test --test big_test_plan tc_001_launch_standalone

# Listar todos sem rodar
cargo test --test big_test_plan -- --list
