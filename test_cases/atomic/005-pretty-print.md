# TC-161 a TC-185: Pretty Printing (Atomic)

---

## TC-161: Pretty print de `value` em `insert`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> print value
```
**Esperado**: Formatação legível de `i32`

---

## TC-162: Pretty print de `root` em `insert`
```bash
rde> break insert
rde> continue
rde> print root
```
**Esperado**: Estrutura de Option<Box<TreeNode>>

---

## TC-163: Pretty print de `root` com sub-árvore
```bash
rde> break insert
rde> continue
rde> print root
```
**Esperado**: Mostra value, left, right, height

---

## TC-164: Pretty print de `root` vazio (None)
```bash
rde> break insert
rde> continue
rde> print root
```
**Esperado**: `None`

---

## TC-165: Pretty print de `node` em `search`
```bash
rde> break search
rde> continue
rde> print node
```
**Esperado**: Option<Box<TreeNode>>

---

## TC-166: Pretty print de `node` em `delete`
```bash
rde> break delete
rde> continue
rde> print node
```
**Esperado**: &mut Option<Box<TreeNode>>

---

## TC-167: Pretty print de `node` em `find_min`
```bash
rde> break find_min
rde> continue
rde> print node
```
**Esperado**: &Option<Box<TreeNode>>

---

## TC-168: Pretty print de `node` em `height`
```bash
rde> break height
rde> continue
rde> print node
```
**Esperado**: &Option<Box<TreeNode>>

---

## TC-169: Pretty print de `node` em `inorder_traversal`
```bash
rde> break inorder_traversal
rde> continue
rde> print node
```
**Esperado**: &Option<Box<TreeNode>>

---

## TC-170: Pretty print de `node` em `preorder_traversal`
```bash
rde> break preorder_traversal
rde> continue
rde> print node
```
**Esperado**: &Option<Box<TreeNode>>

---

## TC-171: Pretty print de `node` em `postorder_traversal`
```bash
rde> break postorder_traversal
rde> continue
rde> print node
```
**Esperado**: &Option<Box<TreeNode>>

---

## TC-172: Pretty print de `tree` (BinaryTree)
```bash
rde> break main
rde> continue
rde> print tree
```
**Esperado**: Estrutura BinaryTree

---

## TC-173: Pretty print de `tree.root` completo
```bash
rde> break main
rde> continue
rde> print tree.root
```
**Esperado**: Árvore completa

---

## TC-174: Pretty print de `tree.size`
```bash
rde> break main
rde> continue
rde> print tree.size
```
**Esperado**: `usize`

---

## TC-175: Pretty print de referência `&value`
```bash
rde> break insert
rde> continue
rde> print &value
```
**Esperado**: Referência com valor

---

## TC-176: Pretty print de `Box<TreeNode>`
```bash
rde> break insert
rde> continue
rde> print *root
```
**Esperado**: TreeNode desreferenciado

---

## TC-177: Pretty print de Option aninhado
```bash
rde> break insert
rde> continue
rde> print root.left
```
**Esperado**: `Some(Box<TreeNode>)` ou `None`

---

## TC-178: Pretty print de enum
**Nota**: Se houver enums no código

---

## TC-179: Pretty print de struct aninhada
```bash
rde> break insert
rde> continue
rde> print root
```
**Esperado**: TreeNode com todos os campos

---

## TC-180: Pretty print de slice/array
**Nota**: Se houver arrays

---

## TC-181: Pretty print com limite de profundidade
```bash
rde> break insert
rde> continue
rde> print root (depth=2)
```
**Esperado**: Trunca após 2 níveis

---

## TC-182: Pretty print com limite de largura
**Nota**: Depende de config

---

## TC-183: Pretty print de Option::None
```bash
rde> break insert
rde> continue
rde> print root.left
```
**Esperado**: `None`

---

## TC-184: Pretty print de Option::Some
```bash
rde> break insert
rde> continue
rde> print root
```
**Esperado**: `Some(TreeNode { ... })`

---

## TC-185: Pretty print após modificação de estrutura
```bash
rde> break insert
rde> continue
rde> print root
rde> continue
rde> print root
```
**Esperado**: Estrutura diferente
