# Agenda — Branch 007-features-left-behind

**Data**: 2026-06-02  
**Branch**: `007-features-left-behind`  
**Spec**: [specs/007-features-left-behind/spec.md](specs/007-features-left-behind/spec.md)  
**Objetivo**: Remover todos os 90 `#[ignore]` de `tests/big_test_plan.rs` e garantir que `cargo test --test big_test_plan` passe com 0 ignorados e 0 falhados (SC-007).

---

## Status Geral

| Área | Status |
|------|--------|
| `#[ignore]` removidos do test plan | ✅ 0 tags restantes em `tests/big_test_plan.rs` |
| Build do workspace | ✅ `cargo build --workspace` passa limpo |
| Suíte de testes completa (SC-007) | 🔄 **EM ANDAMENTO** — última execução: ~249 passados / ~42 falhando |

---

## O Que Foi Feito Nesta Sessão

### 1. Correções de Step Over / Step Out

**Problema**: `tc_083`–`tc_091` (step over/out em `rotate_left`, `rotate_right`, `find_min`, `find_max`) falhavam porque os testes usavam args de demo incorretos — funções nunca eram chamadas, então breakpoints nunca disparavam.

**Solução**: Corrigidos args de demo nos testes (`tests/big_test_plan.rs`):
- `tc_083`/`tc_084`: `&[]` → `&["--demo", "insert-sequence"]` (rotações explícitas no demo)
- `tc_085`: `"insert-sequence"` → `"delete-rebalance"` (`find_min` é chamado no `delete` de 2 filhos)
- `tc_086`: `"insert-sequence"` → `"stress-test"` (`find_max` é chamado no stress-test)
- `tc_090`/`tc_091`: `&[]` → `&["--demo", "insert-sequence"]`

### 2. StackWalk64 — Callbacks Obrigatórios em x64

**Problema**: `StepOut` (comando `finish`) falhava após entrar no prólogo de uma função porque `read_return_address` lia `[RSP]`, que é inválido após `push rbp` / `sub rsp, N`. Além disso, `StackWalk64` em `rde-symbols` passava `None` para os callbacks obrigatórios em x64, fazendo a stack walk falhar silenciosamente.

**Solução**:
- `crates/rde-symbols/src/dbghelp.rs`: carregadas `SymFunctionTableAccess64` e `SymGetModuleBase64`; passadas como callbacks para `StackWalk64`
- Corrigidas assinaturas de callback em `StackWalk64Fn` (estavam erradas no código original)
- `crates/rde-core/src/lib.rs`: `DebugBackend::read_return_address` agora recebe `thread_id` em vez de `rsp`
- `crates/rde-win32/src/step.rs`: `read_return_address` usa `SymbolEngine::walk_stack` para obter o endereço de retorno real via unwind info

### 3. Race Condition no Engine — `tokio::select!` Desbalanceado

**Problema**: `tc_077`, `tc_078`, `tc_098` (múltiplos `step` seguidos de `bt`/`regs`) falhavam intermitentemente com `lines.is_empty()`. A causa raiz: o engine usava `tokio::select!` sem prioridade entre `command_rx` e `debug_event_rx`. Quando um comando de execução (ex: `step`) chegava logo após um debug event (`SingleStep`) mas antes do engine processar o evento, o comando era processado primeiro com `process_state = Running`, rejeitado com erro, e o prompt nunca aparecia corretamente.

**Solução**: `crates/rde-core/src/engine.rs`: `tokio::select! { biased; ... }` com `debug_event_rx` como primeira branch, garantindo que eventos de debug sejam sempre processados antes de comandos quando ambos estão pendentes.

### 4. Tracing Barulhento Interferindo em Testes

**Problema**: logs `tracing::info!` do `debug_loop.rs` e `engine.rs` poluíam o stdout capturado pelo harness de teste, causando falsos negativos em asserções que buscavam "Thread" ou "Breakpoint".

**Mitigação**: logs diagnosticamente úteis permanecem, mas o nível padrão do `FmtSubscriber` em `rde-cli` continua em `INFO`. Os testes que falham por capturar logs de tracing precisam ser revisados caso ainda ocorram (ex: `tc_040`).

---

## Problemas Abertos (Bloqueadores Atuais)

### Categoria: Symbol Resolution
- `tc_029`: `balance_factor` não existe como símbolo no debuggee
- `tc_053`: `clear` não existe como símbolo
- `tc_054`: `is_empty` — símbolo existe mas `break is_empty` não encontra (possível conflito de nome ou inlining)

### Categoria: Attach / Launch Edge Cases
- `tc_008`: attach a processo em execução — attach não retorna prompt corretamente
- `tc_013`: launch de executável inexistente — saída de erro não contém string esperada

### Categoria: Thread Commands
- `tc_040`: `breakpoint_shows_thread_id` — breakpoint hit não mostra "Thread" no output esperado (possível interferência de logs de tracing)

### Categoria: Backtrace / Pretty-Print (não implementados)
- `tc_155`, `tc_157`, `tc_160`, `tc_220`: `bt` retorna "Backtrace not yet implemented"
- `tc_161`–`tc_163`, `tc_165`, `tc_285`: pretty-print não retorna output esperado

### Categoria: Step Into Específicos
- `tc_077`/`tc_078`/`tc_098`: ainda flakam em paralelo; individualmente passam após o fix do `biased` select. Possível solução: reduzir número de steps nos testes ou adicionar sync explícita.

### Categoria: Edge Cases Diversos
- `tc_035`/`tc_037`/`tc_199`: delete all breakpoints / dynamic remove/delete durante pause
- `tc_114`: continue after delete
- `tc_197`: empty command
- `tc_256`: e2e
- `tc_273`: print nonexistent
- `golden_path_demo_success`: golden path

---

## Próximos Passos

1. **Rodar suíte completa** (`cargo test --test big_test_plan`) para obter contagem exata de pass/fail após as correções desta sessão
2. **Fixar symbol tests restantes** — adicionar `#[inline(never)]` em funções faltantes ou ajustar asserts dos testes
3. **Decidir sobre backtrace/pretty-print** — implementar ou marcar como `#[ignore]` com justificativa se estiverem fora do escopo
4. **Limpar código temporário** — remover `eprintln!` de debug inseridos em `tests/big_test_plan.rs`
5. **Verificação final**: `cargo test --test big_test_plan` → 291 TCs, 0 falhados, 0 ignorados

---

## Arquivos Modificados Nesta Sessão

- `tests/big_test_plan.rs` — fix de args de demo em tc_083–tc_091; `eprintln!` de diagnóstico (a remover)
- `crates/rde-core/src/lib.rs` — assinatura de `read_return_address` mudou para `thread_id`
- `crates/rde-core/src/engine.rs` — `tokio::select! { biased; }` priorizando debug events; `read_return_address` usa `tid`
- `crates/rde-win32/src/lib.rs` — adaptação para novo `read_return_address`
- `crates/rde-win32/src/step.rs` — `read_return_address` reescrito com `walk_stack`
- `crates/rde-symbols/src/dbghelp.rs` — fix de `StackWalk64` callbacks + carregamento de `SymFunctionTableAccess64` / `SymGetModuleBase64`
