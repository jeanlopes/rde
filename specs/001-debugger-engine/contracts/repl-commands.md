# Contract: REPL Commands

**Feature**: 001-debugger-engine
**Date**: 2026-05-28

---

## Command Grammar

Todos os comandos são textuais, terminados por newline (`\n`).

```
<command> ::= <launch>
           |  <attach>
           |  <continue>
           |  <step>
           |  <breakpoint>
           |  <delete-breakpoint>
           |  <registers>
           |  <read-memory>
           |  <write-memory>
           |  <backtrace>
           |  <threads>
           |  <modules>
           |  <disassemble>
           |  <quit>

<launch>          ::= "launch" <path> [ <arg> ... ]
<attach>          ::= "attach" <pid>
<continue>        ::= "continue" | "c"
<step>            ::= "step" | "s"
<breakpoint>      ::= "break" <address-or-symbol>
<delete-breakpoint> ::= "delbreak" <breakpoint-id>
<registers>       ::= "regs" | "registers"
<read-memory>     ::= "x" <address> [ <count> ]
<write-memory>    ::= "setmem" <address> <byte> [ <byte> ... ]
<backtrace>       ::= "bt" | "backtrace"
<threads>         ::= "threads" | "info threads"
<modules>         ::= "modules" | "info modules"
<disassemble>     ::= "disas" <address> [ <count> ]
<quit>            ::= "quit" | "q" | "exit"

<address-or-symbol> ::= <hex-address> | <identifier>
<hex-address>     ::= "0x" <hex-digit>+
<pid>             ::= <digit>+
<breakpoint-id>   ::= <digit>+
<address>         ::= <hex-address>
<count>           ::= <digit>+
<byte>            ::= <hex-pair>
<hex-pair>        ::= <hex-digit> <hex-digit>
<path>            ::= <string>
<arg>             ::= <string>
<identifier>      ::= <alpha> [ <alpha> | <digit> | "_" ]*
```

---

## Response Format

Cada comando produz uma resposta no REPL. Respostas são strings terminadas por `\n`.

### Success Responses

| Command | Success Response Pattern |
|---|---|
| `launch` | `Processo iniciado: {path} (PID: {pid})` |
| `attach` | `Anexado ao processo {pid}` |
| `continue` | `Executando...` (ou nada se já rodando) |
| `step` | `Step completo em 0x{address}` |
| `break` | `Breakpoint {id} definido em {symbol} (0x{address})` |
| `delbreak` | `Breakpoint {id} removido.` |
| `regs` | Tabela de registradores (ver formatter) |
| `x` | Hex dump com ASCII sidebar |
| `setmem` | `Memória escrita em 0x{address} ({n} bytes)` |
| `bt` | Lista numerada de frames |
| `threads` | Lista numerada de threads |
| `modules` | Lista de módulos carregados |
| `disas` | Instruções assembly com endereços e bytes |
| `quit` | `Sessão encerrada.` |

### Error Responses

Todas as respostas de erro começam com `Erro:`.

| Condition | Error Response |
|---|---|
| Comando inválido | `Erro: comando desconhecido: {input}` |
| Sessão não iniciada | `Erro: nenhuma sessão ativa. Use 'launch' ou 'attach'.` |
| Endereço inválido | `Erro: endereço inválido: {address}` |
| Breakpoint duplicado | `Erro: já existe breakpoint em 0x{address}` |
| Breakpoint não encontrado | `Erro: breakpoint {id} não encontrado` |
| Processo não acessível | `Erro: acesso negado ao processo {pid}` |
| Memória não legível | `Erro: não foi possível ler memória em 0x{address}` |

---

## Async Event Notifications

Eventos gerados pela engine são impressos no REPL automaticamente (não em resposta a comandos):

| Event | Notification Pattern |
|---|---|
| Breakpoint hit | `[Breakpoint {id}] Hit em {symbol} (0x{address}) — Thread {thread_id}` |
| Single step | `[Step] 0x{address}` |
| Process exited | `[Processo encerrado] código: {exit_code}` |
| Thread created | `[Thread] Criada: {thread_id}` |
| Thread exited | `[Thread] Encerrada: {thread_id}` |
| Module loaded | `[Módulo] Carregado: {name} em 0x{base}` |
| Exception | `[Exceção] {code} em 0x{address}` |
