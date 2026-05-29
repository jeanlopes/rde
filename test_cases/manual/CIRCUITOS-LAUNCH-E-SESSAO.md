# Circuitos: Launch, Session & Durabilidade de Sessão

Cada circuito aqui é uma **sessão longa e contínua** no debugger. Não pare entre os passos — execute tudo em sequência no mesmo terminal REPL.

---

## CIRCUITO-01: Sessão Completa — Launch, Break, Continue, Inspeção, Término

**Objetivo**: Verificar que o debugger completa um ciclo inteiro sem degradação de estado.
**Duração estimada**: 2 minutos
**Pré-condições**: `rde-cli` e `rust_app_example` compilados.

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
```

| Passo | Comando REPL | O que validar |
|-------|-------------|---------------|
| 1 | *(aguarde)* | `Processo iniciado: PID <n>` apareceu? Prompt `rde>` está ativo? |
| 2 | `break main` | Breakpoint ID 1 retornado? |
| 3 | `continue` | Hit em `main`: `[Breakpoint 1] Hit em 0x<addr> — Thread <tid>` |
| 4 | `print value` | Erro ou output? (main não tem `value`, deve dar erro ou lixo) |
| 5 | `regs` | RAX, RBX, RCX, RDX, RIP, RSP visíveis? |
| 6 | `bt` | `#0 main` visível? |
| 7 | `step` | Avançou para `demo`? Prompt retornou? |
| 8 | `bt` | Agora `#0 demo` → `#1 main`? Frame adicionado? |
| 9 | `step` | Avançou para dentro de `insert`? |
| 10 | `bt` | `#0 insert` → `#1 demo` → `#2 main`? |
| 11 | `print value` | `50` (primeiro insert)? |
| 12 | `print root` | `None` (antes de criar raiz)? |
| 13 | `step` | Criou o nó raiz? Avançou? |
| 14 | `print root` | Agora `Some(TreeNode { value: 50, ... })`? |
| 15 | `vars` | `value` e `root` listados? |
| 16 | `continue` | Próximo hit em `insert` (valor 30)? |
| 17 | `print value` | `30`? |
| 18 | `print root.value` | `50`? (raiz existente) |
| 19 | `step` | Entrou na recursão `insert(left)`? |
| 20 | `bt` | 4 frames: `insert` → `insert` → `demo` → `main`? |
| 21 | `print value` | `30` no frame recursivo? |
| 22 | `regs` | RIP mudou? Valores coerentes? |
| 23 | `continue` | Próximo hit (valor 70)? |
| 24 | `print value` | `70`? |
| 25 | `step` | Entrou na recursão right? |
| 26 | `bt` | 4 frames novamente? |
| 27 | `continue` | Hit 20? |
| 28 | `continue` | Hit 40? |
| 29 | `continue` | Hit 60? |
| 30 | `continue` | Hit 80? |
| 31 | `continue` | Processo encerra? |
| 32 | *(sessão encerrou)* | Debugger retornou ao shell sem crash? |

**Observação de resiliência**: Se em qualquer `continue` o debugger hangar (não retornar em 5s), isso é um bug de sincronização.

---

## CIRCUITO-02: Sessão com Múltiplos Launches e Quits

**Objetivo**: Verificar que o debugger limpa estado corretamente entre sessões.
**Duração estimada**: 3 minutos

```bash
# SESSÃO A
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
rde> print value
rde> continue
# Hit seguinte
rde> quit
```

**Validação A**: Sessão A encerrou graciosamente?

```bash
# SESSÃO B (imediatamente após)
rde-cli target/release/rust_app_example.exe --demo search-miss
rde> break search
rde> continue
# Hit
rde> print value
# Deve ser 25
rde> continue
# Falha da busca
rde> quit
```

**Validação B**: Sessão B não carregou breakpoints da sessão A? Estado limpo?

```bash
# SESSÃO C
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
# Hit
rde> print value
# Deve ser 30
rde> continue
rde> quit
```

**Validação C**: Três sessões seguidas sem vazamento de estado?

---

## CIRCUITO-03: Sessão Longa — 100 Inserts com Breakpoints Dinâmicos

**Objetivo**: Testar durabilidade com muitos hits e modificações de breakpoint em runtime.
**Duração estimada**: 5-10 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo stress-test
rde> break insert
rde> continue
```

| Passo | Comando | O que validar |
|-------|---------|---------------|
| 1-10 | `continue` (10×) | 10 hits consecutivos em `insert`. Cada hit responde em < 2s? |
| 11 | `print value` | Valor aleatório mas válido (1~1000)? |
| 12 | `print root` | Árvore crescendo? |
| 13 | `bt` | Pilha de chamadas consistente? |
| 14 | `delbreak 1` | Breakpoint removido |
| 15 | `continue` | Agora NÃO deve haver hit. Processo deve encerrar. |

**Validação de resiliência**: O debugger sobreviveu a 10 hits sem degradação de performance?

---

## CIRCUITO-04: Launch via Cargo com Stale Check Real

**Objetivo**: Verificar que o debugger detecta modificação no fonte e recompila.
**Duração estimada**: 3-5 minutos (inclui build)

```bash
cd examples/rust_app_example

# Compile uma vez
rde-cli cargo debug
rde> quit

# Modifique o fonte
# Edite src/main.rs — adicione um comentário ou mude uma string
# Ex: mude "Demo: insert-sequence" para "Demo: insert-sequence MODIFIED"

# Launch novamente
rde-cli cargo debug
```

| Passo | O que observar |
|-------|---------------|
| 1 | `cargo build` foi disparado automaticamente? |
| 2 | Build completou? |
| 3 | Debug iniciou com o novo binário? |
| 4 | `rde> continue` → output modificado visível? |

**Limpeza**: Reverta a modificação no fonte.

---

## CIRCUITO-05: Sessão com Attach a Processo Running

**Objetivo**: Verificar que attach funciona e o debugger mantém controle.
**Duração estimada**: 3 minutos

**Terminal A** (debuggee):
```bash
target/release/rust_app_example.exe --demo stress-test
# Deixe rodando. Anote o PID no Task Manager ou via:
# tasklist | findstr rust_app
```

**Terminal B** (debugger):
```bash
rde-cli
rde> attach <PID>
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `attach <PID>` | `Processo anexado: PID <n>` |
| 2 | `break insert` | Breakpoint criado |
| 3 | `continue` | Processo continua? |
| 4 | *(aguarde alguns segundos)* | O debuggee está rodando? |
| 5 | `Ctrl+C` ou comando equivalente no REPL | Consegue parar o processo? |
| 6 | `threads` | Threads listadas? |
| 7 | `modules` | Módulos listados? |
| 8 | `detach` ou `quit` | Desanexa graciosamente? |

**Terminal A**: O debuggee continuou rodando após detach?

---

## Checklist de Execução

- [ ] CIRCUITO-01: Sessão Completa — Launch, Break, Continue, Inspeção, Término
- [ ] CIRCUITO-02: Sessão com Múltiplos Launches e Quits
- [ ] CIRCUITO-03: Sessão Longa — 100 Inserts com Breakpoints Dinâmicos
- [ ] CIRCUITO-04: Launch via Cargo com Stale Check Real
- [ ] CIRCUITO-05: Sessão com Attach a Processo Running
