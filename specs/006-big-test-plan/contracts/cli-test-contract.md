# Contract: CLI-Test Harness Interface

**Feature**: 006-big-test-plan
**Date**: 2026-05-28

---

## Purpose

Define o contrato entre o harness de teste de integração e o `rde-cli` para garantir que os testes possam parsear o output do REPL de forma robusta.

---

## Prompt Contract

| Item | Format | Example |
|------|--------|---------|
| Ready prompt | `rde>` | Indica que o REPL está pronto para receber comandos |

**Invariant**: O harness deve enviar comandos apenas após detectar o prompt `rde>` no stdout (ou após um evento que sinalize pausa, como `BreakpointHit`).

---

## Event Output Contract

### Process Lifecycle

| Event | Output Pattern | Example |
|-------|---------------|---------|
| `ProcessLaunched` | `Processo iniciado: PID {pid}` | `Processo iniciado: PID 1234` |
| `ProcessAttached` | `Processo anexado: PID {pid}` | `Processo anexado: PID 1234` |
| `ProcessExited` | `Processo encerrado com código: {code}` | `Processo encerrado com código: 0` |

### Breakpoint Events

| Event | Output Pattern | Example |
|-------|---------------|---------|
| `BreakpointHit` (user) | `[Breakpoint {id}] Hit em 0x{addr:x} — Thread {tid}` | `[Breakpoint 1] Hit em 0x140001000 — Thread 5678` |
| `BreakpointHit` (system) | (implícito, tratado internamente) | — |
| `SingleStep` | `Single step em 0x{addr:x} — Thread {tid}` | `Single step em 0x140001005 — Thread 5678` |

### Exception Events

| Event | Output Pattern | Example |
|-------|---------------|---------|
| `Exception` | `Exceção 0x{code:08X} em 0x{addr:x}` | `Exceção 0xC0000005 em 0x140001200` |

### Thread/Module Events

| Event | Output Pattern | Example |
|-------|---------------|---------|
| `ThreadCreated` | `Thread criada: {tid}` | `Thread criada: 5678` |
| `ThreadExited` | `Thread encerrada: {tid} (código: {code})` | `Thread encerrada: 5678 (código: 0)` |
| `ModuleLoaded` | `Módulo carregado: {name} em 0x{base:x}` | `Módulo carregado: kernel32.dll em 0x7FF812340000` |

---

## Command Response Contract

### Execution Control

| Command | Success Response | Error Response |
|---------|-----------------|----------------|
| `continue` / `c` | (nenhuma saída imediata; processo running) | `Erro: nenhum processo ativo` |
| `step` / `s` | (nenhuma saída imediata; aguarda SingleStep event) | `Erro: processo não pausado` |

### Inspection Commands

| Command | Success Response Pattern | Example |
|---------|-------------------------|---------|
| `print <expr>` | `<formatted_value>` | `42` ou `Some(Node { value: 10, ... })` |
| `vars` | Lista de `nome = valor` | `value = 42` |
| `regs` | Tabela de registradores | `RAX: 0x000000000000002A` |
| `x <addr> <size>` | Hex dump + ASCII | `0x7FF123456000: 48 89 5C ...` |
| `bt` | Lista de frames | `#0 main` |
| `threads` | Tabela de threads | `1234 Suspended *` |
| `modules` | Tabela de módulos | `kernel32.dll 0x7FF8...` |
| `tasks` | Tabela de tasks ou mensagem | `No Tokio runtime detected.` |

### Breakpoint Commands

| Command | Success Response | Error Response |
|---------|-----------------|----------------|
| `break <symbol>` | `Breakpoint {id} definido em {symbol}` | `Erro: símbolo não encontrado` |
| `break <addr>` | `Breakpoint {id} definido em 0x{addr:x}` | `Erro: endereço inválido` |
| `delbreak <id>` | `Breakpoint {id} removido.` | `Erro: breakpoint não encontrado` |

### Configuration Commands

| Command | Success Response | Error Response |
|---------|-----------------|----------------|
| `set pretty-print on\|off` | (nenhuma; afeta comandos futuros) | `Erro: valor inválido` |
| `set print-limit <n>` | (nenhuma) | `Erro: valor inválido` |
| `set print-depth <n>` | (nenhuma) | `Erro: valor inválido` |
| `set auto-disassemble on\|off` | (nenhuma) | `Erro: valor inválido` |
| `set disassembly-count <n>` | (nenhuma) | `Erro: valor inválido` |

---

## Pretty-Print Contract

| Rust Type | Expected Output Pattern | Example |
|-----------|------------------------|---------|
| `i32` | Decimal integer | `42` |
| `i64` | Decimal integer | `9007199254740992` |
| `bool` | `true` / `false` | `true` |
| `String` | Quoted string | `"hello"` |
| `Vec<T>` | Bracketed list | `[10, 20, 30]` |
| `Option<T>::Some` | `Some(...)` | `Some(42)` |
| `Option<T>::None` | `None` | `None` |
| `Result<T,E>::Ok` | `Ok(...)` | `Ok(42)` |
| `Result<T,E>::Err` | `Err(...)` | `Err("fail")` |
| `Box<T>` | Conteúdo dereferenciado | `Node { value: 10 }` |
| `&T` | Conteúdo referenciado | `Node { value: 10 }` |
| Custom struct `TreeNode` | Named fields | `TreeNode { value: 10, left: None, right: None }` |
| Truncated | `...` ou `Truncated` | `...` |

---

## Invariants

1. **Prompt Consistency**: Após cada evento que pausa o processo (breakpoint hit, single step, exception), o prompt `rde>` deve aparecer antes de qualquer comando ser aceito.
2. **Event Ordering**: `ProcessLaunched` (ou `ProcessAttached`) sempre precede `ProcessExited`.
3. **Breakpoint ID Uniqueness**: Cada `break` bem-sucedido retorna um ID numérico único e crescente dentro da sessão.
4. **Thread ID Consistency**: O `tid` em `BreakpointHit` deve corresponder a uma thread listada em `threads`.
5. **No Output During Run**: Enquanto o processo está running (após `continue`), o REPL não deve emitir output até que um evento de pausa ocorra (exceto possíveis mensagens de log/debug).

---

## Versioning

Este contrato é válido para a versão do rde-cli compatível com as specs 001-005. Se o formato do REPL mudar, este contrato deve ser atualizado e os testes devem ser revisados.
