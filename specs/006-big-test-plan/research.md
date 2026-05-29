# Research: Big Test Plan — rde-cli with Binary Tree Debuggee

**Date**: 2026-05-28
**Feature**: 006-big-test-plan

---

## Decision: Debuggee Type — BST with Height Tracking

**Decision**: Implementar uma Binary Search Tree (BST) clássica com tracking de altura, mas sem rebalanceamento automático (não é AVL completa).

**Rationale**:
- Uma BST pura já gera múltiplos caminhos de execução (left vs right) e recursão, essenciais para testar `step into`, `step over`, e backtraces.
- A função `height` adiciona recursão profunda sem complicar a lógica de inserção.
- Funções de rotação (`rotate_left`, `rotate_right`) podem ser expostas como métodos públicos para teste, mas não são chamadas automaticamente — isso permite testar `step into` nelas sem depender de lógica de balanceamento complexa.
- Uma AVL completa adicionaria complexidade desnecessária ao debuggee; o objetivo é testar o debugger, não implementar uma estrutura de dados de produção.

**Alternatives considered**:
- AVL completa: Rejeitada porque a lógica de balanceamento obscureceria os testes de debugger com muitos detalhes algorítmicos.
- Red-Black Tree: Rejeitada pelo mesmo motivo, e adiciona estados complexos (cores dos nós) sem benefício para os testes de debugger.
- Lista ligada simples: Rejeitada porque não exercita recursão nem múltiplos caminhos de execução (não há left/right branch).

---

## Decision: Test Harness Architecture — Subprocess + Pipe

**Decision**: O harness de teste spawna `rde-cli` como subprocesso e interage via stdin/stdout usando pipes.

**Rationale**:
- É o método mais próximo da interação real do usuário com o CLI.
- Não requer modificações no rde-cli (ex: não precisa de modo "testable" ou JSON output).
- Permite testar o REPL real, incluindo prompts, formatação de output e eventos.

**Alternatives considered**:
- Linkar contra `rde-core` diretamente: Rejeitada porque não testaria a camada CLI (argument parsing, REPL loop, formatação de output).
- Usar biblioteca de terminal emulation (ex: `pexpect`): Rejeitada porque adiciona dependência Python (proibida pela constituição).
- Mock do backend: Rejeitada porque o objetivo é testar o engine real com um processo real.

---

## Decision: Golden Path Normalization — Regex Substitution

**Decision**: Normalizar PIDs, TIDs e endereços hex em golden paths usando regex replace por placeholders (`<PID>`, `<TID>`, `<ADDR>`).

**Rationale**:
- Endereços de memória e IDs de processo/thread são deterministicamente imprevisíveis entre execuções devido ao ASLR e alocação dinâmica do SO.
- A estrutura do event stream (ordem e tipos de eventos) deve ser idêntica; apenas os valores dinâmicos variam.
- `insta` suporta filtros de snapshot que permitem esta normalização.

**Alternatives considered**:
- Comparar apenas tipos de eventos (ignorar todos os valores): Rejeitada porque perderia a validação de thread IDs em eventos e endereços de breakpoints.
- Fixar endereços via linker scripts: Rejeitada porque é extremamente complexo e não-portável.

---

## Decision: Test Organization — Single Integration Test File

**Decision**: Todos os 290 casos de teste residem em um único arquivo `tests/src/big_test_plan.rs` com funções individuais `tc_001` a `tc_290`.

**Rationale**:
- Facilita execução seletiva (`cargo test tc_042`).
- Centraliza o harness e helpers compartilhados.
- Documentação clara: cada função tem um comentário doc com o ID, cenário e critério de sucesso copiado da spec.

**Alternatives considered**:
- Separar em múltiplos arquivos por categoria: Rejeitada porque 290 funções em um arquivo são gerenciáveis em Rust (sem problemas de compilação) e a separação dificultaria a localização do harness.
- Usar macro para gerar testes dinamicamente: Rejeitada porque perderia a clareza de ter uma função nomeada por caso de teste.

---

## Decision: Debuggee Location — `examples/rust_app_example/`

**Decision**: Colocar o debuggee em `examples/rust_app_example/` como membro do workspace.

**Rationale**:
- Localização padrão do Cargo para exemplos que são projetos completos.
- Permite compilar com `cargo build --example rust_app_example` se configurado como example, ou `cargo build -p rust_app_example` se configurado como workspace member.
- Mantém separação clara entre o engine e o alvo de teste.

**Alternatives considered**:
- `crates/rust_app_example/`: Rejeitada porque o debuggee não é uma library crate reutilizável; é um aplicativo de teste.
- `test_data/binaries/`: Rejeitada porque dificulta o desenvolvimento iterativo do debuggee (sem `cargo build` direto).

---

## Decision: REPL Event Capture — stdout Line Parsing

**Decision**: O harness captura stdout do rde-cli linha a linha e parseia padrões conhecidos.

**Rationale**:
- O REPL atual do rde-cli é textual. Cada evento (`ProcessLaunched`, `BreakpointHit`, etc.) tem um formato de output específico documentado nas guidelines.
- Não requer alterações no rde-cli.
- Permite validação tanto de eventos estruturados quanto de pretty-printed values.

**Known limitation**: Se o output do REPL mudar (ex: mensagens traduzidas, formatação alterada), os testes quebrarão. Isso é aceitável porque a spec define o formato esperado.

---

## Open Questions (Resolvidos na Plan Phase)

| Question | Resolution |
|----------|------------|
| Como testar `step out` se o engine ainda não suporta? | Marcar TCs de step out como `#[ignore = "step out not implemented"]` até a feature estar disponível. |
| Como testar `tasks` sem Tokio no debuggee? | Validar que a saída é "No Tokio runtime detected." — isso é o comportamento correto esperado. |
| Como garantir que o debuggee compila antes dos testes? | Usar `build.rs` ou `cargo test` dependency: compilar `rust_app_example` como pre-build step. |
