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
| Suíte de testes completa (SC-007) | 🔄 **EM ANDAMENTO** — última execução: **~287 passados / ~5 falhando** |

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

### 5. Backtrace Symbol Resolution + `walk_stack_from_context`

**Problema**: `tc_157`, `tc_160`, `tc_220` falhavam porque o backtrace mostrava "??" para todos os frames. O `resolve(address)` (usado pelo `walk_stack`) só tentava `SymFromAddrW` (que falhava silenciosamente porque DbgHelp não tinha símbolos carregados), e o fallback do PDB buscava apenas endereço **exato** — mas PCs de stack walk apontam para o meio de funções.

**Solução**:
- `crates/rde-symbols/src/dbghelp.rs`: fallback do `resolve()` agora faz busca por **endereço mais próximo menor ou igual** (`addr <= address`), com heurística de distância máxima (64KB)
- Corrigida conversão UTF-16→UTF-8 no `resolve()`: `SymFromAddrW` escreve UTF-16LE no campo `name` do `SYMBOL_INFO` mesmo passando a struct ANSI; detectado pelo padrão de bytes nulos e convertido com `String::from_utf16_lossy`
- Corrigido truncamento: `name_len` é contagem de WCHARs, então lemos `name_len * 2` bytes

### 6. `walk_stack` com Contexto Guardado (Engine)

**Problema**: `walk_stack` chamava `OpenThread` + `GetThreadContext` internamente, que falhava com `ERROR_INVALID_HANDLE` em ~50% das execuções (thread já em estado de transição após o engine modificar contexto para single-step). Além disso, quando o processo single-stepped automaticamente após um breakpoint hit, o `get_context` no momento do `bt` retornava RIP incorreto (em código do sistema).

**Solução**:
- `crates/rde-core/src/engine.rs`: adicionado `Session::last_context: HashMap<u32, RegisterContext>`. Quando o engine processa um `BreakpointHit`, guarda o `RegisterContext` modificado (RIP = endereço do BP, RFLAGS |= TF) antes de continuar
- `crates/rde-core/src/lib.rs`: novo método `walk_stack_with_context(rip, rbp)` no trait `DebugBackend`
- `crates/rde-win32/src/lib.rs`: `walk_stack` agora usa `thread::get_context` (caminho conhecido por funcionar) e passa RIP/RBP para `walk_stack_from_context`; implementado `walk_stack_with_context`
- `crates/rde-core/src/engine.rs`: handler de `Backtrace` usa `walk_stack_with_context` com o contexto guardado se disponível, evitando re-obter contexto de um thread que já foi resumido

### 7. `selected_thread` no System Initial Breakpoint

**Problema**: `tc_220` (`bt` sem nenhum comando prévio) falhava com "No thread selected" porque o engine não setava `selected_thread` quando o system initial breakpoint era atingido.

**Solução**: `crates/rde-core/src/engine.rs`: adicionado `session.selected_thread = Some(thread_id)` no handler de `BreakpointKind::SystemInitial`.

### 8. `resolve_by_name` — Substring Muito Permissivo

**Problema**: `tc_199` falhava porque `break insert` criava **dois** breakpoints (endereços `0x...1570` e `0x...1af0`). O fallback do PDB fazia busca substring genérica (`sym_name.contains("insert")`), que encontrava múltiplos símbolos (`insert`, `insert_left`, instâncias monomorfizadas, etc.).

**Solução**: `crates/rde-symbols/src/dbghelp.rs`: fallback de substring agora prefere matches que terminam com `::{name}` ou são exatamente `{name}`; só cai em substring genérico se não houver nenhum suffix/exact match.

---

## Problemas Abertos (Bloqueadores Atuais)

### Categoria: Attach / Launch Edge Cases
- `tc_008`: attach a processo em execução — attach não retorna prompt corretamente
- `golden_path_demo_success`: golden path — processo termina mas evento `ProcessExited` não aparece no output esperado

### Categoria: Step Sequencing
- `tc_103`/`tc_104`: step-over / step-out sequence — `next`/`finish` travam (processo nunca retorna após step command). Possível interação entre Trap Flag residual e temp breakpoint de step-over causando loop de single-steps.

### Categoria: Backtrace no System Breakpoint
- `tc_220`: backtrace `main` thread — ainda trava ou retorna frames sem `main`. O RBP chain não é confiável no system breakpoint (ntdll sem frame pointers). Precisa de abordagem híbrida StackWalk64 + RBP chain, ou skip de frame walking para endereços de sistema.

---

## Próximos Passos

1. **Investigar step sequencing** (`tc_103`/`tc_104`) — o engine parece entrar em loop de single-steps quando `next` é enviado após um user breakpoint. O Trap Flag pode não estar sendo limpo corretamente no contexto guardado, ou o temp BP de step-over está colidindo com o single-step automático do breakpoint hit.
2. **Fixar tc_220** — implementar fallback seguro no `walk_stack` para system breakpoint (retornar apenas frame 0 ou usar StackWalk64 para código do sistema).
3. **Fixar attach / golden_path** — verificar race conditions no `ProcessExited` detection e no attach flow.
4. **Limpar código temporário** — remover `println!`/`eprintln!` de diagnóstico inseridos em `dbghelp.rs`, `engine.rs`, `thread.rs`.
5. **Verificação final**: `cargo test --test big_test_plan` → 291 TCs, 0 falhados, 0 ignorados

---

## Arquivos Modificados Nesta Sessão

- `tests/big_test_plan.rs` — `eprintln!` de diagnóstico (a remover)
- `crates/rde-core/src/lib.rs` — novo método `walk_stack_with_context` no trait `DebugBackend`
- `crates/rde-core/src/engine.rs` — `last_context` no `Session`; `Backtrace` usa contexto guardado; `selected_thread` setado no system breakpoint
- `crates/rde-win32/src/lib.rs` — `walk_stack` usa `thread::get_context` + `walk_stack_from_context`; `walk_stack_with_context` implementado
- `crates/rde-symbols/src/dbghelp.rs` — `walk_stack_from_context` (RBP chain); `resolve` com busca aproximada e conversão UTF-16→UTF-8; `resolve_by_name` com suffix match preferencial
