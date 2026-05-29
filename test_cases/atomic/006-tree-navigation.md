# TC-186 a TC-210: Tree Navigation (Atomic)

---

## TC-186: backtrace em `main`
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
rde> bt
```
**Esperado**: 1 frame: main

---

## TC-187: backtrace em `insert` profundidade 1
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> bt
```
**Esperado**: main → insert

---

## TC-188: backtrace em `insert` profundidade 2
```bash
rde> break insert
rde> continue
rde> step
rde> bt
```
**Esperado**: main → insert → insert

---

## TC-189: backtrace em `insert` profundidade 5
```bash
rde> break insert
rde> continue
rde> step (5x)
rde> bt
```
**Esperado**: 7 frames

---

## TC-190: backtrace em `search` profundidade 1
```bash
rde-cli target/release/rust_app_example.exe --demo search-miss
rde> break search
rde> continue
rde> bt
```
**Esperado**: main → search

---

## TC-191: backtrace em `delete` profundidade 1
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
rde> bt
```
**Esperado**: main → delete

---

## TC-192: backtrace em `find_min`
```bash
rde> break find_min
rde> continue
rde> bt
```
**Esperado**: main → delete → find_min

---

## TC-193: backtrace em `find_max`
```bash
rde> break find_max
rde> continue
rde> bt
```
**Esperado**: main → find_max

---

## TC-194: backtrace em `height`
```bash
rde> break height
rde> continue
rde> bt
```
**Esperado**: main → height → height ...

---

## TC-195: backtrace em `inorder_traversal`
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break inorder_traversal
rde> continue
rde> bt
```
**Esperado**: main → inorder_traversal → inorder_traversal ...

---

## TC-196: backtrace em `preorder_traversal`
```bash
rde> break preorder_traversal
rde> continue
rde> bt
```
**Esperado**: main → preorder ...

---

## TC-197: backtrace em `postorder_traversal`
```bash
rde> break postorder_traversal
rde> continue
rde> bt
```
**Esperado**: main → postorder ...

---

## TC-198: backtrace em `rotate_left`
```bash
rde> break rotate_left
rde> continue
rde> bt
```
**Esperado**: main → rotate_left

---

## TC-199: backtrace em `rotate_right`
```bash
rde> break rotate_right
rde> continue
rde> bt
```
**Esperado**: main → rotate_right

---

## TC-200: backtrace após step out
**Nota**: Requer step out

---

## TC-201: backtrace com múltiplos frames
```bash
rde> break insert
rde> continue
rde> step
rde> step
rde> step
rde> bt
```
**Esperado**: 5+ frames

---

## TC-202: Navegação de frame: frame 0
```bash
rde> break insert
rde> continue
rde> bt
rde> frame 0
rde> print value
```
**Esperado**: frame 0, valor correto

---

## TC-203: Navegação de frame: frame 1
```bash
rde> break insert
rde> continue
rde> bt
rde> frame 1
rde> print value
```
**Esperado**: frame 1, valor do chamador

---

## TC-204: Navegação de frame: frame -1 (último)
```bash
rde> break insert
rde> continue
rde> bt
rde> frame -1
```
**Esperado**: Último frame

---

## TC-205: Navegação de frame: frame inválido
```bash
rde> break insert
rde> continue
rde> frame 99
```
**Esperado**: Erro

---

## TC-206: Print em frame diferente do atual
```bash
rde> break insert
rde> continue
rde> step
rde> frame 0
rde> print value
rde> frame 1
rde> print value
```
**Esperado**: Valores diferentes

---

## TC-207: Step em frame diferente
**Nota**: Comportamento depende da implementação

---

## TC-208: Continue mantém frame selecionado
**Nota**: Comportamento depende da implementação

---

## TC-209: backtrace em demo `stress-test`
```bash
rde-cli target/release/rust_app_example.exe --demo stress-test
rde> break insert
rde> continue
rde> bt
```
**Esperado**: Stack com profundidade variável

---

## TC-210: backtrace em recursão profunda (nível 10)
```bash
rde> break insert
rde> continue
rde> step (10x)
rde> bt
```
**Esperado**: 12 frames
