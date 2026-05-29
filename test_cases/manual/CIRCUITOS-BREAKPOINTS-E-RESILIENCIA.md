# Circuitos: Breakpoints, Resiliência e Estado Acumulado

Cada circuito aqui cria **estado complexo de breakpoints** e testa se o debugger se mantém estável após dezenas de operações de breakpoint.

---

## CIRCUITO-10: Maratona de Breakpoints — Criar, Usar, Deletar, Recriar

**Objetivo**: Testar que o manager de breakpoints não corrompe estado após muitas operações.
**Duração estimada**: 5 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `break main` | ID 1 |
| 2 | `break insert` | ID 2 |
| 3 | `break search` | ID 3 |
| 4 | `break delete` | ID 4 |
| 5 | `break find_min` | ID 5 |
| 6 | `break find_max` | ID 6 |
| 7 | `break height` | ID 7 |
| 8 | `break size` | ID 8 |
| 9 | `break inorder_traversal` | ID 9 |
| 10 | `continue` | Hit no primeiro breakpoint atingível (provavelmente `main` ou `insert`) |
| 11 | `continue` | Próximo hit |
| 12 | `delbreak 5` | Remove `find_min` |
| 13 | `continue` | Ainda hitando nos demais? |
| 14 | `break find_min` | Recria com novo ID (10?) |
| 15 | `delbreak 1` | Remove `main` |
| 16 | `delbreak 2` | Remove `insert` |
| 17 | `delbreak 3` | Remove `search` |
| 18 | `delbreak 4` | Remove `delete` |
| 19 | `delbreak 6` | Remove `find_max` |
| 20 | `delbreak 7` | Remove `height` |
| 21 | `delbreak 8` | Remove `size` |
| 22 | `delbreak 9` | Remove `inorder_traversal` |
| 23 | `delbreak 10` | Remove `find_min` recriado |
| 24 | `continue` | NENHUM hit. Processo encerra. |

**Observação de resiliência**: Se após deletar e recriar breakpoints o hit falhar, o manager de breakpoints tem bug de reutilização de slots.

---

## CIRCUITO-11: Breakpoint em Tempo de Execução — Adicionar Durante Continue

**Objetivo**: Testar que breakpoints adicionados enquanto o processo está running são ativados corretamente.
**Duração estimada**: 3 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break main
rde> continue
# Hit em main
rde> continue
# Agora o processo está running (executando inserts)
# TENTE adicionar breakpoint durante execução:
rde> break insert
```

| Passo | O que validar |
|-------|---------------|
| 1 | O comando `break insert` foi aceito enquanto running? |
| 2 | Se aceito, o processo parou no próximo `insert`? |
| 3 | Se rejeitado, qual foi a mensagem? (Esperado: "processo running" ou similar) |

**Variação**: Se o REPL não aceita comandos durante running, teste após um hit:
```bash
rde> break main
rde> continue
# Hit em main
rde> break insert  # Adicionado enquanto pausado
rde> continue      # Deve hitar em insert agora
```

---

## CIRCUITO-12: Breakpoint Duplicado e Remoção Seletiva

**Objetivo**: Testar que breakpoints duplicados funcionam independentemente.
**Duração estimada**: 2 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> break insert
rde> break insert
# 3 breakpoints no mesmo símbolo
rde> continue
```

| Passo | Validação |
|-------|-----------|
| 1 | 3 IDs diferentes retornados? |
| 2 | Hit no primeiro `insert` — quantos reports aparecem? (Esperado: 1, com um dos IDs) |
| 3 | `delbreak 2` — remove o segundo |
| 4 | `continue` — ainda hitando? |
| 5 | `delbreak 1` |
| 6 | `delbreak 3` |
| 7 | `continue` — processo encerra sem hits |

---

## CIRCUITO-13: System Initial Breakpoint + User Breakpoint em Sequência

**Objetivo**: Verificar que o system initial breakpoint não interfere com user breakpoints subsequentes.
**Duração estimada**: 2 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | *(launch)* | Processo iniciado e pausado (system breakpoint) |
| 2 | `continue` | Sai do system breakpoint, executa até... |
| 3 | `break insert` | Cria user breakpoint |
| 4 | `continue` | Hit em `insert` — VERIFICAR que não é confundido com system breakpoint |
| 5 | `step` | Single step funciona? |
| 6 | `continue` | Próximo hit em `insert` |
| 7 | `continue` | Próximo hit... |
| 8 | *(repita)* | Todos os 7 inserts devem ser interceptados |

**Observação**: Se o debugger entrar em loop infinito (breakpoint → step → breakpoint) após o system initial, isso é o bug clássico de confusão system-vs-user.

---

## CIRCUITO-14: Breakpoint em Toda Função da Árvore — Full Coverage

**Objetivo**: Navegar por todas as funções da BST com breakpoints.
**Duração estimada**: 4 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break insert
rde> break search
rde> break delete
rde> break find_min
rde> break find_max
rde> break height
rde> break size
rde> break inorder_traversal
rde> break preorder_traversal
rde> break postorder_traversal
rde> break is_empty
rde> break clear
rde> continue
```

| Passo | Validação |
|-------|-----------|
| 1 | Hit na primeira função executada |
| 2 | `continue` → próxima função |
| 3 | Repita até o fim |
| 4 | Verifique se TODAS as funções da árvore foram hitadas |
| 5 | Se alguma função NÃO foi hitada, verifique se é porque não foi chamada pela demo |

---

## Checklist de Execução

- [ ] CIRCUITO-10: Maratona de Breakpoints — Criar, Usar, Deletar, Recriar
- [ ] CIRCUITO-11: Breakpoint em Tempo de Execução — Adicionar Durante Continue
- [ ] CIRCUITO-12: Breakpoint Duplicado e Remoção Seletiva
- [ ] CIRCUITO-13: System Initial Breakpoint + User Breakpoint em Sequência
- [ ] CIRCUITO-14: Breakpoint em Toda Função da Árvore — Full Coverage
