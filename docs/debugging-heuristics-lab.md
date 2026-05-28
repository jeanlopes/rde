# Laboratório de Heurísticas de Debugging — RDE

> Sessão de autópsia e propostas de melhoria após resolução de três bugs críticos no debugger Windows.
> Data: 2026-05-28

---

## 1. Autópsia dos Três Bugs: Onde Tempo Foi Jogado Fora

### Bug 1 — Thread Affinity (`WaitForDebugEvent`)

**O que demorou:** Horas procurando em timeouts, permissões, flags de criação, e handles inválidos.
**O que era:** Uma invariante arquitetural não-documentada do Win32.

**Fraqueza identificada:** Fui "ajustando variáveis" (timeout, flags, ordem de chamada) em vez de primeiro perguntar: *"qual é o contrato exato da API que estou usando?"* O MSDN diz claramente que `WaitForDebugEvent` deve rodar na mesma thread que `CreateProcessW` com `DEBUG_PROCESS` — mas li a assinatura, não o contrato. Tentei resolver por **indução** (testar combinações) em vez de **dedução** (ler o contrato).

**Custo:** Alto. Cada tentativa exigia rebuild, run, observar comportamento, formular nova hipótese.

---

### Bug 2 — Indentação / Timeout Handling

**O que demorou:** Procurar no engine, no handler de breakpoints, nas flags de continue, nas threads.
**O que era:** Um `dispatch_event` fora do bloco `if result.is_ok()`.

**Fraqueza identificada:** Quando vi `ERROR_INVALID_HANDLE (6)`, a heurística foi: *"handle está sendo usado depois de fechado"* ou *"estou passando PID em vez de thread ID"*. Nunca se considerou: *"o struct está sendo lido quando a API falhou"*. Isso é **falta de invariante documentada**: se houvesse um comentário no código dizendo *"DEBUG_EVENT só é válido se WaitForDebugEvent retornar Ok"*, a indentação errada teria gritado aos olhos.

**Custo:** Médio-Alto. O bug era visualmente óbvio, mas invisível sem o contrato mental correto.

---

### Bug 3 — Loop de Breakpoint Desconhecido

**O que demorou:** Entender o loop infinito breakpoint→single-step→breakpoint, e por que RIP decrementado + TF não resolvia.
**O que era:** O Windows injeta um `int3` inicial em `ntdll` que **não é nosso** — e a regra é: `DBG_CONTINUE` sem tocar em registers.

**Fraqueza identificada:** Assumiu-se que todo `EXCEPTION_BREAKPOINT` deveria ser tratado pelo handler de breakpoints (restaurar byte, decrementar RIP, setar TF). Nunca se perguntou: *"e se o breakpoint não for nosso?"* Foi **dono da verdade** sobre o domínio (Win32 debugging) em vez de primeiro mapear: *"quais breakpoints especiais o sistema gera automaticamente?"*

**Custo:** Alto. Cada ciclo do loop gerava logs, e os logs pareciam "quase certos", criando confusão.

---

## 2. Padrão das Fraquezas: O "Debugger Anti-Pattern"

| Bug | Heurística Ruim | Heurística que Funcionaria |
|-----|-----------------|---------------------------|
| Thread affinity | Ajustar variáveis e testar | Documentar contrato da API antes de usar |
| Indentação | Procurar em outras camadas | Verificar invariantes de controle de fluxo |
| Breakpoint loop | Assumir uniformidade de eventos | Mapear eventos do sistema vs. eventos do usuário |

O problema não é velocidade. É que **não havia um modelo mental compartilhado** sobre o que o Win32 garante vs. o que nosso código assume.

---

## 3. Proposta: O "Contrato-Primeiro" para Debuggers

### 3.1. Documentação de Contratos de API (`docs/contracts/`)

Criar um arquivo por API crítica, formato fixo:

```markdown
# Contract: WaitForDebugEventEx

## Pre-conditions
- Thread calling this MUST be the same thread that called CreateProcessW with DEBUG_PROCESS.
- Process handle must be valid and open.

## Post-conditions (Ok)
- `event` struct is fully populated.
- Process is suspended until ContinueDebugEvent is called.

## Post-conditions (Err)
- `event` struct is UNDEFINED (may be zeroed, may be garbage).
- MUST NOT call dispatch_event or ContinueDebugEvent with this struct.

## Timeouts
- ERROR_SEM_TIMEOUT (0x102): normal, ignore silently.
- After EXIT_PROCESS: may get 0x80070079, also ignore.
```

**Por que isso encurta a busca:** Quando um erro aparece, a primeira pergunta não é *"o que está errado no nosso código?"* mas sim: *"qual invariante do contrato está sendo violada?"*

---

### 3.2. Invariantes no Código (não só nos docs)

Comentários no código que são **assertivos**, não descritivos:

```rust
// INVARIANT: event is only valid if WaitForDebugEventEx returned Ok.
// VIOLATION: calling dispatch_event on timeout caused ERROR_INVALID_HANDLE.
match unsafe { WaitForDebugEventEx(&mut event, 100) } {
    Ok(()) => {
        // SAFE: event is populated here.
        let needs_continue = dispatch_event(&event, &event_tx);
        // ...
    }
    Err(e) => {
        // SAFE: event is UNINITIALIZED here. Do NOT touch it.
    }
}
```

Isso transforma bugs de "lógica" em bugs de "visibilidade". A indentação errada do Bug 2 teria sido óbvia se o comentário `// SAFE` estivesse no lugar errado.

---

### 3.3. Logger de Estado Estruturado (não só de erros)

Em vez de:
```rust
warn!("WaitForDebugEventEx failed: error=0x{:08X}", code);
```

Usar:
```rust
trace!(
    target: "rde::debug_loop::state",
    ?event,
    ?result,
    ?running,
    "WaitForDebugEventEx returned"
);
```

E mais importante: **logar transições de estado, não só erros**. O bug do loop de breakpoint só foi entendido quando se viu a *sequência* de eventos. Precisamos de:

```rust
// No engine:
info!(
    target: "rde::engine::transition",
    from = ?current_state,
    to = ?new_state,
    event = ?event,
    "state transition"
);
```

Isso permite diffs entre uma run boa e uma run ruim.

---

### 3.4. Golden Path / Snapshot Testing

Rodar `rust_app_example.exe --demo` uma vez com sucesso, capturar a sequência exata de eventos:

```rust
// test_data/demo_golden_path.txt
[0ms] ProcessLaunched { pid: 1234 }
[10ms] BreakpointHit { id: 0, address: 0x7ff... }  // system initial
[10ms] Continue
[15ms] BreakpointHit { id: 1, address: 0x14... }   // user main
...
[500ms] ProcessExited { code: 0 }
```

Nos próximos runs, comparar contra esse golden path. A primeira divergência aponta exatamente onde a regressão começou.

---

### 3.5. Separação Semântica de Eventos no Type System

O Bug 3 aconteceu porque `BreakpointHit` era um tipo único para dois conceitos diferentes:

```rust
enum BreakpointKind {
    SystemInitial,  // Windows ntdll breakpoint — DBG_CONTINUE, no register touch
    UserDefined(u32), // Our breakpoint — restore byte, dec RIP, set TF
}
```

Se o tipo do compilador nos obriga a distinguir, não podemos "esquecer" de tratar o caso do sistema.

---

## 4. Checklist de Debugging para Próximos Bugs

Quando algo falhar no futuro, seguir esta ordem **estrita**:

1. **É timeout ou evento real?**
   → Se timeout, o struct está indefinido. Nada mais deve acontecer.

2. **É evento do sistema ou do usuário?**
   → Se do sistema (initial breakpoint, DLL load), não aplicar lógica de usuário.

3. **A thread certa está executando?**
   → Verificar thread ID da criadora do processo vs. thread do debug loop.

4. **Qual a sequência de estado esperada vs. observada?**
   → Olhar os últimos 5 eventos no log, não só o erro atual.

5. **Qual invariante do contrato está sendo violada?**
   → Checar `docs/contracts/` antes de mudar código.

---

## 5. Próximos Passos (Action Items)

- [ ] Criar `docs/contracts/win32-debug-api.md` com contratos das APIs críticas.
- [ ] Adicionar comentários de invariante em `debug_loop.rs`, `engine.rs`, e `process.rs`.
- [ ] Implementar `BreakpointKind` no type system para separar breakpoints de sistema.
- [ ] Adicionar logging de transição de estado no engine.
- [ ] Criar golden path / snapshot do `--demo`.

---

> Transformar o projeto de "vamos ver se funciona" para "sabemos exatamente por que funciona, e o próximo bug será encontrado em 1/10 do tempo".
