# Implementation Plan: Disassembly View

**Branch**: `003-disassembly-view` | **Date**: 2026-05-28 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-disassembly-view/spec.md`

## Summary

Adicionar visualização de código assembly ao RDE via comando `disassemble` (alias `disas`), usando Capstone Engine para decodificação x86-64. O feature inclui: disassembly centrado no RIP do thread selecionado, disassembly em endereços/símbolos arbitrários, destaque de breakpoints ativos (`b+`), e modo `auto-disassemble` que reemite o view automaticamente após paradas (`BreakpointHit`, `SingleStep`, `Exception`).

**Technical approach**: Novo módulo `disasm` em `rde-core` que encapsula o Capstone, lê memória do debuggee via `ReadProcessMemory` (exposto por `rde-win32`), consulta breakpoints ativos via `rde-breakpoint`, e formata o output como texto puro para o REPL. O parser do REPL ganha novos comandos e configurações (`set auto-disassemble`, `set disassembly-count`).

## Technical Context

**Language/Version**: Rust stable (MSRV 1.78, declarado em `workspace.package`)

**Primary Dependencies**: `capstone` 0.12 (disassembly x86-64), `tokio`, `tracing`, `windows-rs` 0.56

**Storage**: N/A — estado temporário em memória durante o comando (DisassemblyView descartado após renderização)

**Testing**: `cargo test`, `insta` para snapshots do output formatado, testes de integração com debuggee de exemplo

**Target Platform**: Windows 10/11 x86-64

**Project Type**: CLI debugger (workspace de crates: `rde-core`, `rde-win32`, `rde-symbols`, `rde-breakpoint`, `rde-repl`, `rde-cli`)

**Performance Goals**: Disassembly ao redor do RIP em <1s; resolução de símbolo por nome em <500ms

**Constraints**: `unsafe` apenas em `rde-win32` (3-condition proof obrigatório); leitura de memória limitada a `min(64 × count, 4096)` bytes por chamada; output textual puro (TUI post-MVP)

**Scale/Scope**: Processos debuggee de tamanho médio; até ~100 instruções visíveis por comando

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| I. Engine-First | ✅ PASS | Disassembly é query direta ao estado do processo via Win32 `ReadProcessMemory`; não usa wrapper externo |
| II. Crate-First | ✅ PASS | Capacidade adicionada como módulo em `rde-core` (coeso com engine) + parser em `rde-repl`; não cria binário monolítico |
| III. REPL-First | ✅ PASS | Comando `disassemble` transita via `EngineCommand` → canal async; não bloqueia o debug loop |
| IV. Windows-Native Purity | ✅ PASS | Capstone em modo x86-64; sem abstrações multi-plataforma |
| V. Reference-Driven | ✅ PASS | Padrão de disassembly em debuggers Win32 é bem estabelecido; nenhuma cópia de C++ |
| VI. Rust Safety | ✅ PASS | `unsafe` apenas em `rde-win32` para `ReadProcessMemory` e `GetThreadContext`; contrato documentado |
| VII. Hackable by Design | ✅ PASS | ~14 FRs, 4 user stories, 5 critérios de sucesso; cabe em uma tarde de leitura |
| VIII. Contract-First | ✅ PASS | Contrato `ReadProcessMemory` criado em `docs/contracts/`; invariants de memória e handle explícitos |

**Re-check after Phase 1**: ✅ Sem alterações — design mantém compliance.

## API Contracts & Invariants

| API / Boundary | Pre-conditions | Post-conditions (Ok) | Post-conditions (Err) | Invariant |
|----------------|---------------|----------------------|-----------------------|-----------|
| `ReadProcessMemory` | `hProcess` é handle válido com `PROCESS_VM_READ`; `lpBaseAddress` alinhado a byte (x86-64 não exige alinhamento de instrução) | Buffer preenchido com bytes lidos; `lpNumberOfBytesRead` indica quantidade real | Buffer conteúdo indefinido; `GetLastError` indica causa (acesso negado, página não commitada, etc.) | Nunca ler mais de 4096 bytes por chamada de disassembly (FR-013) |
| `GetThreadContext` | `hThread` é handle válido com `THREAD_GET_CONTEXT`; thread está suspensa ou processo parado | `CONTEXT` preenchido com registradores, incluindo `Rip` | `CONTEXT` conteúdo indefinido; `GetLastError` indica causa | Sempre usar `WOW64_CONTEXT` ou `CONTEXT` dependendo do target; aqui apenas x64 nativo |
| Capstone (`capstone::Capstone::disasm_all`) | Buffer de bytes válido; modo `x86-64` configurado | Retorna `Vec<Insn>` com instruções decodificadas | Retorna erro de decodificação (ex: dados inválidos) | Capstone é stateless entre chamadas; não mantém referência ao buffer após retorno |
| Breakpoint Manager query | Sessão ativa; breakpoint manager inicializado | Retorna `HashSet<u64>` de endereços ativos | Erro se manager não inicializado | Endereços são virtuais addresses do processo debuggee |

**Type-system distinctions:**
- [x] Este feature NÃO introduz ambiguidade sistema-vs-usuário — disassembly é uma query de leitura, não um evento do debug loop.
- [x] Não usa magic values — RIP é um `u64` normal; thread selecionada é `Option<u32>`.

**Golden path:**
- Event sequence esperado para P1 (Disassemble at RIP on Breakpoint Hit):
  1. Debuggee executa até INT3 em endereço com breakpoint
  2. `WaitForDebugEventEx` retorna `EXCEPTION_DEBUG_EVENT` (INT3)
  3. Engine mapeia para `EngineEvent::BreakpointHit { address, thread_id }`
  4. REPL recebe evento; se `auto-disassemble on`, envia `EngineCommand::Disassemble { address: None, thread_id }`
  5. Engine lê memória em `RIP - offset` via `ReadProcessMemory`
  6. Engine disassembla com Capstone
  7. Engine consulta breakpoints ativos, marca `b+` e `=>`
  8. Engine retorna `EngineEvent::Output(disassembly_text)`
  9. REPL renderiza e exibe prompt
- Snapshot será armazenado em: `test_data/golden_paths/003_disassembly_rip.txt`

## Project Structure

### Documentation (this feature)

```text
specs/003-disassembly-view/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── read-process-memory.md
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── rde-core/
│   ├── src/
│   │   ├── lib.rs            # Re-exporta disasm types
│   │   ├── events.rs         # Adiciona EngineCommand::Disassemble, EngineEvent::Output
│   │   ├── engine.rs         # Handler de Disassemble no Session
│   │   └── disasm.rs         # NEW: Disassembler, DisassemblyLine, DisassemblyView, formatação
│   └── Cargo.toml            # Adiciona dependency: capstone = "0.12"
├── rde-win32/
│   ├── src/
│   │   ├── lib.rs            # Re-exporta ReadProcessMemory wrapper
│   │   ├── process.rs        # ReadProcessMemory wrapper (já existe? verificar)
│   │   └── thread.rs         # GetThreadContext / RIP extraction
│   └── Cargo.toml            # Sem mudanças
├── rde-breakpoint/
│   ├── src/
│   │   └── lib.rs            # Adiciona método list_active() -> HashSet<u64>
│   └── Cargo.toml            # Sem mudanças
├── rde-symbols/
│   ├── src/
│   │   └── lib.rs            # Adiciona resolve_by_name() -> Result<Vec<u64>, _>
│   └── Cargo.toml            # Sem mudanças
├── rde-repl/
│   ├── src/
│   │   ├── lib.rs            # Auto-disassemble após eventos de parada
│   │   ├── parser.rs         # Parsing de `disas`, `set auto-disassemble`, `set disassembly-count`
│   │   └── commands.rs       # (se existir) Dispatch de comandos
│   └── Cargo.toml            # Sem mudanças
└── rde-cli/
    ├── src/
    │   └── main.rs           # Sem mudanças (orquestração inalterada)
    └── Cargo.toml            # Sem mudanças

tests/
├── integration_tests.rs      # Adicionar teste: disassembly após breakpoint hit

test_data/
└── golden_paths/
    └── 003_disassembly_rip.txt

docs/
└── contracts/
    └── read-process-memory.md
```

**Structure Decision**: Disassembly é uma capability de visualização do engine, não um subsistema independente. Mantido dentro de `rde-core` como módulo `disasm` para coesão com o `Session` e acesso direto a threads/breakpoints. Parser e auto-display ficam em `rde-repl`. Contratos em `docs/contracts/` conforme gate da Constituição.

## Complexity Tracking

> No violations detected. Constitution Check passed cleanly.
