# Circuitos: Execution Control e Navegação Profunda

Cada circuito aqui é uma **jornada de navegação** através da árvore binária, usando step, continue, e inspeção em cada nível.

---

## CIRCUITO-20: Descida Completa — Step Into em Toda a Árvore

**Objetivo**: Descer manualmente por todos os níveis da árvore via step into.
**Duração estimada**: 5 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
```

| Passo | Comando | Onde você deve estar |
|-------|---------|---------------------|
| 1 | `continue` | Hit 1: `insert(50)` — criando raiz |
| 2 | `step` | Dentro de `insert` |
| 3 | `print value` | `50` |
| 4 | `print root` | `None` (ainda) |
| 5 | `step` | Próxima instrução (Box::new?) |
| 6 | `print root` | Agora `Some(...)` |
| 7 | `continue` | Hit 2: `insert(30)` — recursão left |
| 8 | `step` | Dentro da recursão |
| 9 | `print value` | `30` |
| 10 | `print root.value` | `50` (raiz existente) |
| 11 | `step` | Entrando no branch left |
| 12 | `bt` | 4 frames: insert → insert → demo → main |
| 13 | `continue` | Hit 3: `insert(70)` — recursão right |
| 14 | `step` | Dentro da recursão right |
| 15 | `bt` | 4 frames |
| 16 | `continue` | Hit 4: `insert(20)` — 3 níveis de profundidade |
| 17 | `step` | Recursão left de left |
| 18 | `bt` | 5 frames! |
| 19 | `print value` | `20` |
| 20 | `continue` | Hit 5: `insert(40)` |
| 21 | `step` | 4 frames |
| 22 | `continue` | Hit 6: `insert(60)` |
| 23 | `step` | 4 frames |
| 24 | `continue` | Hit 7: `insert(80)` |
| 25 | `step` | 4 frames |
| 26 | `continue` | Processo encerra |

**Observação**: Se em qualquer momento o `step` não avançar ou o prompt não retornar em 3s, há bug no single step.

---

## CIRCUITO-21: Subida e Descida — Step Out e Step Into Alternados

**Objetivo**: Navegar para baixo e para cima na pilha de chamadas.
**Duração estimada**: 4 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit: insert(50)
rde> step
# Dentro de insert
rde> step
# Mais profundo
rde> bt
# 4 frames
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `finish` (ou `step out`) | Subiu um nível. Frame removido? |
| 2 | `bt` | 3 frames? |
| 3 | `step` | Desceu de novo? |
| 4 | `bt` | 4 frames de novo? |
| 5 | `finish` | Subiu |
| 6 | `finish` | Subiu mais |
| 7 | `finish` | Chegou em `demo`? |
| 8 | `bt` | 2 frames? |
| 9 | `continue` | Próximo hit |

**Se `step out` não existir**: Use `continue` e `break` combinados para simular a subida.

---

## CIRCUITO-22: Traversal Completo com Step em Cada Nó

**Objetivo**: Acompanhar inorder traversal passo a passo.
**Duração estimada**: 3 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break inorder_traversal
rde> continue
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `continue` | Hit em `inorder_traversal` |
| 2 | `print result` | `[]` (vazio) |
| 3 | `step` | Entrou em left recursão |
| 4 | `print result` | Ainda `[]` |
| 5 | `step` | Entrou em left de left (10) |
| 6 | `print result` | `[]` — folha, left é None |
| 7 | `step` | Push 10 em result |
| 8 | `print result` | `[10]`! |
| 9 | `step` | Retorna para 20 |
| 10 | `print result` | `[10]` |
| 11 | `step` | Push 20 |
| 12 | `print result` | `[10, 20]`! |
| 13 | `step` | Entra em right (30) |
| 14 | `step` | Push 30 |
| 15 | `print result` | `[10, 20, 30]`! |

---

## CIRCUITO-23: Delete com Três Caminhos Diferentes

**Objetivo**: Testar delete de folha, um filho, e dois filhos na mesma sessão.
**Duração estimada**: 4 minutos

**Nota**: A demo `delete-rebalance` deleta 30 (dois filhos). Para testar os outros casos, você pode criar uma árvore manualmente ou modificar o debuggee.

### Caminho 1: Delete de folha
```bash
rde-cli target/release/rust_app_example.exe
rde> break insert
rde> continue
# Crie uma árvore com folha: insert 50, insert 30
# (modifique o debuggee para pausar após inserções, ou use step)
```

Alternativa: Modifique `main.rs` temporariamente para:
```rust
let mut tree = BinaryTree::new();
tree.insert(50);
tree.insert(30);  // folha
tree.delete(30);  // delete folha
```

**Sessão**:
```bash
rde-cli target/release/rust_app_example.exe
rde> break delete
rde> continue
# Hit em delete(30)
rde> print root.left
# Some(...)
rde> print root.right
# None — confirma: é folha!
rde> step
# Deleta
rde> print root.left
# None — folha removida!
```

### Caminho 2: Delete com um filho
Modifique o debuggee para:
```rust
tree.insert(50);
tree.insert(30);
tree.insert(20); // 30 tem um filho left
tree.delete(30); // delete com um filho
```

**Sessão**:
```bash
rde> break delete
rde> continue
rde> print root.left.left
# Some(20)
rde> print root.left.right
# None — um filho!
rde> step
rde> print root.left.value
# 20 — filho subiu!
```

### Caminho 3: Delete com dois filhos (demo original)
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
rde> print root.left
# Some(20)
rde> print root.right
# Some(40) — dois filhos!
rde> step
# Entra em find_min
rde> bt
# find_min → delete → demo → main
rde> print root.value
# Novo valor (mínimo da direita = 40)
```

---

## CIRCUITO-24: Rotação Manual com Inspeção de Estrutura

**Objetivo**: Verificar que rotate_left/right reestruturam a árvore corretamente.
**Duração estimada**: 2 minutos

Modifique o debuggee temporariamente:
```rust
let mut tree = BinaryTree::new();
tree.insert(50);
tree.insert(30);
tree.insert(70);
tree.rotate_left(); // 70 vira raiz
```

**Sessão**:
```bash
rde-cli target/release/rust_app_example.exe
rde> break rotate_left
rde> continue
rde> print root.value
# Antes: 50
rde> step
# Após rotação
rde> print root.value
# Depois: 70!
rde> print root.left.value
# 50
rde> print root.right
# None (ou o que era filho de 70)
```

---

## Checklist de Execução

- [ ] CIRCUITO-20: Descida Completa — Step Into em Toda a Árvore
- [ ] CIRCUITO-21: Subida e Descida — Step Out e Step Into Alternados
- [ ] CIRCUITO-22: Traversal Completo com Step em Cada Nó
- [ ] CIRCUITO-23: Delete com Três Caminhos Diferentes
- [ ] CIRCUITO-24: Rotação Manual com Inspeção de Estrutura
