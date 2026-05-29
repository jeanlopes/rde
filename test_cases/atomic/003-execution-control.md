# TC-061 a TC-120: Execution Control (Atomic)

---

## TC-061: Continue após breakpoint em `main`
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
# Hit
rde> continue
```
**Esperado**: Processo encerra

---

## TC-062: Continue após breakpoint em `insert`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
rde> continue
```
**Esperado**: Próximo hit ou fim

---

## TC-063: Continue após breakpoint em `delete`
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
# Hit
rde> continue
```
**Esperado**: Próximo hit ou fim

---

## TC-064: Continue sem breakpoint seguinte
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
# Hit
rde> delbreak 1
rde> continue
```
**Esperado**: Processo encerra sem mais hits

---

## TC-065: Step into em `main` chamando `demo`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break main
rde> continue
# Hit
rde> step
```
**Esperado**: Entra em demo

---

## TC-066: Step into em `demo` chamando `insert`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break main
rde> continue
rde> step
rde> step
```
**Esperado**: Entra em insert

---

## TC-067: Step into em `insert` chamando `insert` recursivo
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
rde> step
```
**Esperado**: Desce um nível de recursão

---

## TC-068: Step into em `insert` chamando `rotate_left`
**Nota**: Insert não chama rotate_left na implementação atual.

---

## TC-069: Step into em `insert` chamando `rotate_right`
**Nota**: Insert não chama rotate_right na implementação atual.

---

## TC-070: Step into em `delete` chamando `find_min`
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
rde> step
```
**Esperado**: Entra em find_min

---

## TC-071: Step into em `delete` chamando `delete` recursivo
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
rde> step
```
**Esperado**: Desce um nível de recursão

---

## TC-072: Step into em `search` chamando `search` recursivo
```bash
rde-cli target/release/rust_app_example.exe --demo search-miss
rde> break search
rde> continue
rde> step
```
**Esperado**: Desce um nível

---

## TC-073: Step into em `height` chamando `height` recursivo
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break height
rde> continue
rde> step
```
**Esperado**: Desce um nível

---

## TC-074: Step into em `inorder_traversal` recursivo
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break inorder_traversal
rde> continue
rde> step
```
**Esperado**: Desce um nível

---

## TC-075: Step into em `preorder_traversal` recursivo
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break preorder_traversal
rde> continue
rde> step
```
**Esperado**: Desce um nível

---

## TC-076: Step into em `postorder_traversal` recursivo
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break postorder_traversal
rde> continue
rde> step
```
**Esperado**: Desce um nível

---

## TC-077: Step into em `Box::new`
**Nota**: Comportamento varia (entra no alloc ou step over implícito)

---

## TC-078: Step into em `drop` de `Box<Node>`
**Nota**: Comportamento varia

---

## TC-079: Step over em `main`
**Nota**: Requer comando `step over` ou `next` no rde-cli.
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
rde> next
```
**Esperado**: Avanca uma linha sem entrar em chamadas

---

## TC-080: Step over em `insert`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> next
```
**Esperado**: Avanca uma linha sem entrar em recursao

---

## TC-081: Step over em `search`
```bash
rde-cli target/release/rust_app_example.exe --demo search-miss
rde> break search
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-082: Step over em `delete`
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-083: Step over em `find_min`
```bash
rde> break find_min
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-084: Step over em `find_max`
```bash
rde> break find_max
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-085: Step over em `height`
```bash
rde> break height
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-086: Step over em `inorder_traversal`
```bash
rde> break inorder_traversal
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-087: Step over em `preorder_traversal`
```bash
rde> break preorder_traversal
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-088: Step over em `postorder_traversal`
```bash
rde> break postorder_traversal
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-089: Step over em `rotate_left`
```bash
rde> break rotate_left
rde> continue
rde> next
```
**Esperado**: Avanca uma linha

---

## TC-090: Step out de `insert`
**Nota**: Requer comando `step out` ou `finish` no rde-cli.
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao frame anterior

---

## TC-091: Step out de `search`
```bash
rde> break search
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-092: Step out de `delete`
```bash
rde> break delete
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-093: Step out de `find_min`
```bash
rde> break find_min
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-094: Step out de `find_max`
```bash
rde> break find_max
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-095: Step out de `height`
```bash
rde> break height
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-096: Step out de `inorder_traversal`
```bash
rde> break inorder_traversal
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-097: Step out de `preorder_traversal`
```bash
rde> break preorder_traversal
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-098: Step out de `postorder_traversal`
```bash
rde> break postorder_traversal
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-099: Step out de `rotate_left`
```bash
rde> break rotate_left
rde> continue
rde> step
rde> finish
```
**Esperado**: Retorna ao chamador

---

## TC-100: Continue após step sequence
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> step
rde> step
rde> continue
```
**Esperado**: Prossegue normalmente

---

## TC-101: Step sequence: main → demo → insert → rotate
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break main
rde> continue
rde> step
rde> step
rde> step
```
**Esperado**: Navega pelas funções

---

## TC-102: Step sequence: insert recursivo 5 níveis
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> step
rde> step
rde> step
rde> step
rde> step
```
**Esperado**: 5 frames de recursão

---

## TC-103: Step over sequence em loop de inorder
**Nota**: Requer step over

---

## TC-104: Step out sequence de profundidade 5
**Nota**: Requer step out

---

## TC-105: Mixed control: step, continue, step
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> step
rde> continue
rde> step
```
**Esperado**: Transições corretas

---

## TC-106: Mixed control: continue, step into
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> step
```
**Esperado**: Para no hit, depois step funciona

---

## TC-107: Mixed control: breakpoint hit, step out
**Nota**: Requer step out

---

## TC-108: Mixed control: breakpoint hit, continue
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
rde> continue
```
**Esperado**: Prossegue até próximo

---

## TC-109: Execution control em `main` vazio
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
rde> step
```
**Esperado**: Avança para demo

---

## TC-110: Execution control em função sem chamadas
```bash
rde-cli target/release/rust_app_example.exe
rde> break find_min
rde> continue
rde> step
```
**Esperado**: Avança ou retorna

---

## TC-111: Continue após system initial breakpoint
```bash
rde-cli target/release/rust_app_example.exe
rde> continue
```
**Esperado**: Executa além de ntdll

---

## TC-112: Step após system initial breakpoint
```bash
rde-cli target/release/rust_app_example.exe
rde> step
```
**Esperado**: Single step funciona

---

## TC-113: Continue com múltiplos breakpoints
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break main
rde> break insert
rde> break search
rde> break delete
rde> break height
rde> continue
```
**Esperado**: Para em cada um

---

## TC-114: Continue após deleção de breakpoint ativo
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
rde> delbreak 1
rde> continue
```
**Esperado**: Não re-hita imediatamente

---

## TC-115: Continue após modificação de breakpoint
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
rde> delbreak 1
rde> break insert
rde> continue
```
**Esperado**: Hit no novo endereço

---

## TC-116: Step into com breakpoint no destino
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> break main
rde> continue
# Hit em main
rde> step
```
**Esperado**: Para tanto no step quanto no breakpoint

---

## TC-117: Step over com breakpoint dentro da função chamada
**Nota**: Requer step over

---

## TC-118: Step out com breakpoint no retorno
**Nota**: Requer step out

---

## TC-119: Execution control durante stress test
```bash
rde-cli target/release/rust_app_example.exe --demo stress-test
rde> break insert
rde> continue
```
**Esperado**: Performance aceitável

---

## TC-120: Execution control com processo que panics
**Nota**: Requer debuggee que panic
