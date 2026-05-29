# TC-251 a TC-270: Memory & Disassembly (Atomic)

---

## TC-251: Examinar memória em `main`
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
rde> x/16wx $rip
```
**Esperado**: Dump de memória

---

## TC-252: Examinar memória em `insert`
```bash
rde> break insert
rde> continue
rde> x/16wx $rsp
```
**Esperado**: Stack

---

## TC-253: Examinar memória de `value`
```bash
rde> break insert
rde> continue
rde> x/4wx &value
```
**Esperado**: Bytes de i32

---

## TC-254: Examinar memória de `root`
```bash
rde> break insert
rde> continue
rde> x/32wx &root
```
**Esperado**: Estrutura do nó

---

## TC-255: Disassemble `main`
```bash
rde> break main
rde> continue
rde> disassemble main
```
**Esperado**: Assembly de main

---

## TC-256: Disassemble `insert`
```bash
rde> break insert
rde> continue
rde> disassemble insert
```
**Esperado**: Assembly de insert

---

## TC-257: Disassemble `search`
```bash
rde> break search
rde> continue
rde> disassemble search
```
**Esperado**: Assembly de search

---

## TC-258: Disassemble `delete`
```bash
rde> break delete
rde> continue
rde> disassemble delete
```
**Esperado**: Assembly de delete

---

## TC-259: Disassemble `find_min`
```bash
rde> break find_min
rde> continue
rde> disassemble find_min
```
**Esperado**: Assembly de find_min

---

## TC-260: Disassemble `find_max`
```bash
rde> break find_max
rde> continue
rde> disassemble find_max
```
**Esperado**: Assembly de find_max

---

## TC-261: Disassemble `height`
```bash
rde> break height
rde> continue
rde> disassemble height
```
**Esperado**: Assembly de height

---

## TC-262: Disassemble `inorder_traversal`
```bash
rde> break inorder_traversal
rde> continue
rde> disassemble inorder_traversal
```
**Esperado**: Assembly

---

## TC-263: Disassemble `preorder_traversal`
```bash
rde> break preorder_traversal
rde> continue
rde> disassemble preorder_traversal
```
**Esperado**: Assembly

---

## TC-264: Disassemble `postorder_traversal`
```bash
rde> break postorder_traversal
rde> continue
rde> disassemble postorder_traversal
```
**Esperado**: Assembly

---

## TC-265: Disassemble `rotate_left`
```bash
rde> break rotate_left
rde> continue
rde> disassemble rotate_left
```
**Esperado**: Assembly

---

## TC-266: Disassemble `rotate_right`
```bash
rde> break rotate_right
rde> continue
rde> disassemble rotate_right
```
**Esperado**: Assembly

---

## TC-267: Memory dump com diferentes formatos
```bash
rde> x/16bx $rip
rde> x/16hx $rip
rde> x/16wx $rip
rde> x/16gx $rip
```
**Esperado**: Diferentes tamanhos

---

## TC-268: Disassemble com contagem
```bash
rde> disassemble main 20
```
**Esperado**: 20 instruções

---

## TC-269: Memory dump de endereço inválido
```bash
rde> x/16wx 0x0
```
**Esperado**: Erro de acesso

---

## TC-270: Disassemble de símbolo inválido
```bash
rde> disassemble nonexistent
```
**Esperado**: Erro
