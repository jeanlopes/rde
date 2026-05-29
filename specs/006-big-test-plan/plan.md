# Implementation Plan: Big Test Plan — rde-cli with Binary Tree Debuggee

**Branch**: `006-big-test-plan` | **Date**: 2026-05-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/006-big-test-plan/spec.md`

---

## Summary

Criar uma aplicação exemplo `rust_app_example` (árvore binária de busca em Rust) e implementar 290+ casos de teste de integração que exercitem **todas** as funcionalidades do `rde-cli` via linha de comando. A suite de testes cobre: launch (standalone e cargo), breakpoints (set/delete/dynamic), execution control (continue/step-into/step-over/step-out), inspeção de variáveis (`print`, `vars`, `regs`, `bt`), pretty printing de tipos Rust, comandos REPL em runtime, thread/module inspection, memory/disassembly, e cenários end-to-end completos.

A abordagem técnica é:
1. Criar o debuggee `rust_app_example` como projeto Cargo independente na pasta `examples/rust_app_example/`.
2. Implementar testes de integração em `tests/` que invoquem `rde-cli` como subprocesso, interajam via stdin/stdout e validem saídas contra golden paths.
3. Para casos que exigem interatividade (step, REPL commands), usar um harness de teste que envie comandos sequenciais e capture eventos.

---

## Technical Context

**Language/Version**: Rust 1.78+ (MSRV do workspace)

**Primary Dependencies**: `rde-cli` (binário alvo dos testes), `tokio` (async runtime do harness de teste, se necessário), `insta` (snapshot testing para golden paths), `serde_json` (parse de eventos estruturados se o CLI emitir JSON no futuro). O debuggee usa apenas `std`.

**Storage**: N/A — testes são efêmeros, sem persistência de estado entre execuções.

**Testing**: `cargo test --test integration` no workspace RDE + golden path comparison (`insta` ou diff textual). O harness de teste executa `rde-cli` como processo filho e comunica-se via pipes (stdin/stdout).

**Target Platform**: Windows 10/11 x86-64 (mesmo alvo do RDE).

**Project Type**: CLI tool integration test suite + example debuggee application.

**Performance Goals**: Suite completa (290 casos) executa em menos de 10 minutos em máquina de desenvolvimento. Cada caso de teste individual deve completar em menos de 30 segundos.

**Constraints**: 
- O debuggee NÃO deve depender de Tokio (para manter os testes de `tasks` determinísticos — devem retornar "No Tokio runtime detected").
- O harness de teste deve funcionar sem privilégios elevados (não requer admin).
- Golden paths devem ser estáveis entre execuções (endereços hex dinâmicos devem ser normalizados nos snapshots).

**Scale/Scope**: 
- 1 aplicação exemplo (~300-500 LOC).
- 290 casos de teste documentados, todos automatizáveis.
- 10 user stories cobrindo todas as áreas funcionais do rde-cli.

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| **I. Engine-First, Not Wrapper-First** | ✅ PASS | Esta feature NÃO modifica o engine; ela *testa* o engine existente via CLI. O engine continua sendo nativo Win32. |
| **II. Crate-First Modularity** | ✅ PASS | O debuggee é um projeto Cargo separado. Os testes residem em `tests/` (ou crate de testes dedicada). Nenhuma capability é siloed em binário sem crate correspondente. |
| **III. REPL-First Architecture** | ✅ PASS | Os testes validam a arquitetura REPL-first existente; não a modificam. Comandos são enviados via stdin do REPL e eventos capturados via stdout. |
| **IV. Windows-Native Purity** | ✅ PASS | Target permanece Windows x64. Nenhuma abstração multi-plataforma é introduzida. |
| **V. Reference-Driven Development** | ✅ PASS | O debuggee é implementado do zero em Rust idiomático. Não há C++ copiado. |
| **VI. Rust Safety First — No External Runtimes** | ✅ PASS | Zero dependências proibidas. O debuggee é `std` apenas. O harness de teste usa apenas crates do ecossistema Rust permitido. |
| **VII. Hackable by Design** | ✅ PASS | O debuggee é pequeno e auto-contido. Os testes são documentados em tabelas claras na spec. |
| **VIII. Contract-First Debugging** | ✅ PASS | Os testes verificam contratos existentes (ex: system initial breakpoint recebe `DBG_CONTINUE` apenas). Nenhum novo contrato Win32 é introduzido. |

**Re-evaluação pós-Phase 1**: A estrutura proposta mantém compliance. Nenhuma violação identificada.

---

## API Contracts & Invariants

*Esta feature é puramente de teste e NÃO modifica `crates/rde-win32` ou o engine. No entanto, os testes verificam contratos existentes. Abaixo estão os contratos que os casos de teste validam:*

| API / Boundary | Pre-conditions | Post-conditions (Ok) | Post-conditions (Err) | Invariant |
|----------------|---------------|----------------------|-----------------------|-----------|
| `WaitForDebugEventEx` (validado indiretamente) | Same thread as `CreateProcessW` | `DEBUG_EVENT` valid | `DEBUG_EVENT` undefined | Testes verificam que eventos são recebidos na ordem correta |
| `ContinueDebugEvent` (validado indiretamente) | IDs de `WaitForDebugEventEx` Ok | Thread resumes | — | Testes verificam que `continue` não hanga |
| System Initial Breakpoint | `EXCEPTION_BREAKPOINT` em ntdll | `DBG_CONTINUE` apenas | — | TC-056 valida que system breakpoint NÃO dispara restore byte / dec RIP |
| User Breakpoint (INT3) | Endereço com 0xCC escrito pelo engine | Restore byte, dec RIP, set TF | — | TC-016 a TC-060 validam ciclo completo de hit → restore → step → reinstall |
| REPL Command Dispatch | Processo pausado | Comando processado | Erro se running | TC-274 a TC-277 validam comandos em estado inválido |

**Type-system distinctions:**
- [x] Esta feature NÃO introduz novas ambiguidades system-vs-user. Os testes validam as distinções existentes (ex: TC-056 distingue system initial breakpoint de user breakpoints).
- [x] Não há magic values novos. Testes usam IDs retornados pelo engine.

**Golden path:**
- [x] O cenário primário de sucesso é: `Launch` → `SystemInitial BreakpointHit` → `Continue` → `ModuleLoaded` events → `ProcessExited`.
- [x] Golden path armazenado em: `test_data/golden_paths/006-big-test-plan-demo.txt` (novo) e comparado contra `test_data/golden_paths/demo_success.txt` (existente).

---

## Project Structure

### Documentation (this feature)

```text
specs/006-big-test-plan/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (CLI-test contract)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
examples/rust_app_example/
├── Cargo.toml           # Projeto do debuggee
└── src/
    └── main.rs          # Árvore binária + demo scenarios

tests/
└── src/
    └── big_test_plan.rs # Harness de teste e 290 casos

test_data/golden_paths/
├── demo_success.txt              # Existente
└── 006-big-test-plan-demo.txt    # Novo golden path para esta feature
```

**Structure Decision**: 
- O debuggee fica em `examples/rust_app_example/` (projeto Cargo independente, mas dentro do workspace RDE como `workspace.members` ou referenciado manualmente).
- Os testes de integração ficam em `tests/src/big_test_plan.rs` (fora dos crates individuais, testando o CLI end-to-end).
- Alternativa considerada: colocar o debuggee em `crates/rde-cli/examples/`. Rejeitada porque o debuggee é uma aplicação completa com `Cargo.toml` próprio, não um snippet de exemplo.

---

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

Nenhuma violação identificada. Esta seção permanece vazia.

---

## Phase 0: Research & Unknown Resolution

### Unknowns Resolvidos

| Unknown | Decisão | Rationale |
|---------|---------|-----------|
| Tipo de árvore binária | **BST clássica com altura** (próximo de AVL, mas sem balanceamento obrigatório para simplificar) | Uma BST pura já gera recursão profunda e múltiplos caminhos (left/right). Altura é calculada para ter uma função recursiva interessante. Se necessário, rotações simples podem ser adicionadas como funções auxiliares. |
| Local do debuggee | `examples/rust_app_example/` como projeto Cargo independente | Permite compilar separadamente, usar como alvo para `rde-cli cargo debug`, e manter o workspace RDE limpo. |
| Framework de teste | **Integration tests Rust** + **golden path textual** | `cargo test` já é o padrão do projeto. O harness spawna `rde-cli` e interage via pipes. Snapshots com `insta` ou comparação direta de strings normalizadas. |
| Interatividade com REPL | **Sequencial stdin/stdout** | O harness envia comandos separados por `\n` e lê respostas até prompt `rde>`. Para casos assíncronos (eventos durante continue), usar thread de leitura separada. |
| Normalização de endereços em golden paths | **Regex replace** de PIDs, TIDs e endereços hex por placeholders (`<PID>`, `<ADDR>`) | Endereços são determinísticos em estrutura, mas não em valor absoluto. A normalização permite snapshots estáveis. |

### Decisões Arquiteturais

1. **Debuggee Structure**: `Tree` struct com `root: Option<Box<Node>>`. Cada `Node` tem `value: i32`, `left: Option<Box<Node>>`, `right: Option<Box<Node>>`. Implementa `Display` para visualização textual da árvore.
2. **Demo Scenarios**: A `main` aceita `--demo <name>` onde `<name>` é um dos: `insert-sequence`, `search-miss`, `delete-rebalance`, `full-traversal`, `stress-test`.
3. **Test Harness**: Função auxiliar `run_rde_cli(commands: &[&str]) -> (stdout: String, events: Vec<String>)` que spawna o processo, envia comandos e coleta output.
4. **Event Parsing**: O harness parseia linhas de output do REPL para extrair eventos (`ProcessLaunched`, `BreakpointHit`, etc.) e pretty-printed values.

---

## Phase 1: Design & Contracts

### Data Model

Ver `specs/006-big-test-plan/data-model.md` para detalhes completos.

Resumo:
- **BinaryTree**: Raiz `Option<Box<Node>>`, métodos: `insert`, `search`, `delete`, `find_min`, `find_max`, `height`, `size`, `inorder_traversal`, `preorder_traversal`, `postorder_traversal`, `is_empty`, `clear`.
- **TreeNode**: `value: i32`, `left: Option<Box<TreeNode>>`, `right: Option<Box<TreeNode>>`, `height: i32` (para cálculo de altura).
- **Demo scenarios**: Mapeamento de string CLI para sequências de operações na árvore.

### Contracts

Ver `specs/006-big-test-plan/contracts/cli-test-contract.md`.

Resumo:
- Contrato entre o harness de teste e o CLI: formato do prompt (`rde>`), formato de eventos (`[Breakpoint {id}] Hit em 0x{addr:x} — Thread {tid}`), formato de pretty-print (`Type(value)`), e comandos válidos.
- Este contrato permite que o harness parseie o output de forma robusta.

### Quickstart

Ver `specs/006-big-test-plan/quickstart.md`.

Resumo:
- Como compilar o debuggee: `cd examples/rust_app_example && cargo build`.
- Como executar um caso de teste: `cargo test --test big_test_plan tc_001`.
- Como atualizar golden paths: `cargo test --test big_test_plan -- --bless` (ou equivalente com `INSTA_UPDATE=always`).
- Como depurar um teste falhando: `RUST_LOG=trace cargo test --test big_test_plan tc_001 -- --nocapture`.

---

## Phase 2: Task Breakdown (Preparatório)

*Esta seção é um esboço para `/speckit-tasks`. Os tasks detalhados serão gerados pelo comando `/speckit-tasks`.*

Tarefas principais:
1. Criar projeto `examples/rust_app_example/` com BST e demo scenarios.
2. Implementar harness de teste em `tests/src/big_test_plan.rs`.
3. Implementar TC-001 a TC-060 (Launch + Breakpoints).
4. Implementar TC-061 a TC-120 (Execution Control).
5. Implementar TC-121 a TC-160 (Variable Inspection).
6. Implementar TC-161 a TC-185 (Pretty Printing).
7. Implementar TC-186 a TC-210 (REPL Runtime).
8. Implementar TC-211 a TC-230 (Thread/Module/Task).
9. Implementar TC-231 a TC-250 (Memory/Disassembly).
10. Implementar TC-251 a TC-270 (E2E Integration).
11. Implementar TC-271 a TC-290 (Error Handling).
12. Criar/validar golden paths.
13. Documentar e revisar.
