# Circuitos: Inspeção de Estado e Pretty Printing em Sessões Longas

Cada circuito aqui mantém uma **sessão contínua** onde você inspeciona múltiplas variáveis em múltiplos contextos, testando se o pretty printer se mantém estável.

---

## CIRCUITO-30: Inspeção Completa de uma Inserção — De None a Árvore de 3 Níveis

**Objetivo**: Acompanhar a evolução da árvore via `print` durante uma sessão.
**Duração estimada**: 4 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
```

| Passo | Comando | Estado esperado da árvore |
|-------|---------|--------------------------|
| 1 | `continue` | Hit: insert(50) |
| 2 | `print root` | `None` |
| 3 | `print value` | `50` |
| 4 | `continue` | Hit: insert(30) |
| 5 | `print root` | `Some(TreeNode { value: 50, ... })` |
| 6 | `print root.left` | `None` (ainda) |
| 7 | `print root.right` | `None` |
| 8 | `continue` | Hit: insert(70) |
| 9 | `print root.left` | `Some(TreeNode { value: 30, ... })` |
| 10 | `print root.right` | `None` (ainda) |
| 11 | `continue` | Hit: insert(20) |
| 12 | `print root.left.left` | `None` (ainda) |
| 13 | `continue` | Hit: insert(40) |
| 14 | `print root.left.right` | `None` (ainda) |
| 15 | `continue` | Hit: insert(60) |
| 16 | `print root.right.left` | `None` |
| 17 | `continue` | Hit: insert(80) |
| 18 | `print root.right.right` | `None` |
| 19 | `continue` | Processo encerra |
| 20 | `quit` | Debugger fecha |

**Variação de resiliência**: Execute `print root` 20× seguidos durante a sessão. O output deve ser consistente — se falhar ou corromper após N prints, há bug no pretty printer.

---

## CIRCUITO-31: Pretty Print com Configurações Dinâmicas

**Objetivo**: Mudar configurações de pretty print durante a sessão e observar efeitos imediatos.
**Duração estimada**: 3 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo stress-test
rde> break insert
rde> continue
# Aguarde até a árvore ter ~50 nós
# (dê continue 50× ou espere o hit 50)
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `continue` (×50) | Árvore com 50+ nós |
| 2 | `set print-depth 1` | Configura |
| 3 | `print root` | Apenas 1 nível expandido |
| 4 | `set print-depth 3` | Configura |
| 5 | `print root` | 3 níveis expandidos |
| 6 | `set print-depth 10` | Configura |
| 7 | `print root` | Árvore quase completa |
| 8 | `set print-limit 5` | Configura |
| 9 | `print result` (se houver Vec) | Apenas 5 elementos |
| 10 | `set print-limit 500` | Configura |
| 11 | `print result` | 500 elementos |
| 12 | `set pretty-print off` | Desliga pretty print |
| 13 | `print root` | Output raw/hex (não estruturado) |
| 14 | `set pretty-print on` | Liga pretty print |
| 15 | `print root` | Output estruturado de volta |

**Observação de resiliência**: Se após mudar `pretty-print off` → `on` o output permanecer raw, há bug de toggle.

---

## CIRCUITO-32: Vars, Regs, BT em Cada Frame de Recursão

**Objetivo**: Inspecionar estado completo (variáveis, registradores, backtrace) em cada nível de recursão.
**Duração estimada**: 5 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit: insert(50)
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `vars` | `value`, `root` listados |
| 2 | `regs` | Registradores x86-64 |
| 3 | `bt` | 3-4 frames |
| 4 | `continue` | Hit: insert(30) |
| 5 | `vars` | `value=30`, `root` aponta para 50 |
| 6 | `regs` | RIP diferente do passo 2? |
| 7 | `bt` | 4-5 frames (recursão!) |
| 8 | `continue` | Hit: insert(70) |
| 9 | `vars` | `value=70` |
| 10 | `regs` | RIP diferente novamente |
| 11 | `bt` | 4-5 frames |
| 12 | `continue` | Hit: insert(20) |
| 13 | `vars` | `value=20` |
| 14 | `bt` | 5-6 frames (mais profundo!) |
| 15 | `continue` | Próximo hit... |

**Observação de resiliência**: Se `bt` mostrar frames corrompidos ou `vars` listar variáveis erradas em frames profundos, há bug no unwinding ou no scope resolution.

---

## CIRCUITO-33: Traversal com Pretty Print do Vec Resultado

**Objetivo**: Observar o Vec crescer durante traversal.
**Duração estimada**: 3 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break inorder_traversal
rde> continue
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `print result` | `[]` |
| 2 | `step` | Entra left |
| 3 | `step` | Entra left de left (10) |
| 4 | `step` | Push 10 |
| 5 | `print result` | `[10]` |
| 6 | `step` | Retorna |
| 7 | `step` | Push 20 |
| 8 | `print result` | `[10, 20]` |
| 9 | `step` | Entra right (30) |
| 10 | `step` | Push 30 |
| 11 | `print result` | `[10, 20, 30]` |
| 12 | `continue` | Próximo traversal? |
| 13 | `break preorder_traversal` | Novo breakpoint |
| 14 | `continue` | Hit |
| 15 | `print result` | `[]` (resetado para preorder) |
| 16 | `step` | Push 20 (raiz primeiro em preorder) |
| 17 | `print result` | `[20]` |
| 18 | `continue` | Postorder |
| 19 | `break postorder_traversal` | |
| 20 | `continue` | Hit |
| 21 | `print result` | `[]` |
| 22 | `step` | Push 10 |
| 23 | `print result` | `[10]` |
| 24 | `step` | Push 30 |
| 25 | `print result` | `[10, 30]` |
| 26 | `step` | Push 20 |
| 27 | `print result` | `[10, 30, 20]` |

---

## CIRCUITO-34: Inspeção de Tipos Diversos na Mesma Sessão

**Objetivo**: Testar pretty printer com múltiplos tipos em sequência rápida.
**Duração estimada**: 2 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
```

| Passo | Comando | Tipo testado |
|-------|---------|-------------|
| 1 | `print value` | `i32` |
| 2 | `print root` | `Option<Box<TreeNode>>` |
| 3 | `print true` | `bool` |
| 4 | `print "hello"` | `String` |
| 5 | `print [1, 2, 3]` | `Vec<i32>` (se suportado) |
| 6 | `print Some(42)` | `Option<i32>` |
| 7 | `print None` | `Option<T>` |
| 8 | `print Ok(42)` | `Result<i32, String>` |
| 9 | `print Err("fail")` | `Result<i32, String>` |
| 10 | `print (10, "hello")` | Tupla |
| 11 | `print 9007199254740992i64` | `i64` |
| 12 | `print 3.14` | `f64` |

**Observação de resiliência**: Se o pretty printer falhar em algum tipo e parar de funcionar para os tipos seguintes, há bug de estado corrompido no printer.

---

## Checklist de Execução

- [ ] CIRCUITO-30: Inspeção Completa de uma Inserção
- [ ] CIRCUITO-31: Pretty Print com Configurações Dinâmicas
- [ ] CIRCUITO-32: Vars, Regs, BT em Cada Frame de Recursão
- [ ] CIRCUITO-33: Traversal com Pretty Print do Vec Resultado
- [ ] CIRCUITO-34: Inspeção de Tipos Diversos na Mesma Sessão
