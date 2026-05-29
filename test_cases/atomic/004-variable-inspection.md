# TC-121 a TC-160: Variable Inspection (Atomic)

---

## TC-121: Print valor no início de `main`
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
rde> print value
```
**Esperado**: Mostra valor (se definido)

---

## TC-122: Print `value` no início de `insert`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> print value
```
**Esperado**: `i32` com valor inserido

---

## TC-123: Print `root` no início de `insert`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> print root
```
**Esperado**: `Option<Box<TreeNode>>`

---

## TC-124: Print `root.value`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> print root.value
```
**Esperado**: `i32` ou erro se None

---

## TC-125: Print `root.left`
```bash
rde> break insert
rde> continue
rde> print root.left
```
**Esperado**: `Option<Box<TreeNode>>`

---

## TC-126: Print `root.right`
```bash
rde> break insert
rde> continue
rde> print root.right
```
**Esperado**: `Option<Box<TreeNode>>`

---

## TC-127: Print `root.height`
```bash
rde> break insert
rde> continue
rde> print root.height
```
**Esperado**: `i32`

---

## TC-128: Print `value` em `search`
```bash
rde-cli target/release/rust_app_example.exe --demo search-miss
rde> break search
rde> continue
rde> print value
```
**Esperado**: `i32`

---

## TC-129: Print `node` em `search`
```bash
rde> break search
rde> continue
rde> print node
```
**Esperado**: `&Option<Box<TreeNode>>`

---

## TC-130: Print `node.value`
```bash
rde> break search
rde> continue
rde> print node.value
```
**Esperado**: `i32`

---

## TC-131: Print `value` em `delete`
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
rde> print value
```
**Esperado**: `i32`

---

## TC-132: Print `root` em `delete`
```bash
rde> break delete
rde> continue
rde> print root
```
**Esperado**: `&mut Option<Box<TreeNode>>`

---

## TC-133: Print `value` em `find_min`
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break find_min
rde> continue
rde> print value
```
**Esperado**: `i32`

---

## TC-134: Print `node` em `find_min`
```bash
rde> break find_min
rde> continue
rde> print node
```
**Esperado**: `&Option<Box<TreeNode>>`

---

## TC-135: Print `value` em `find_max`
```bash
rde> break find_max
rde> continue
rde> print value
```
**Esperado**: `i32`

---

## TC-136: Print `node` em `find_max`
```bash
rde> break find_max
rde> continue
rde> print node
```
**Esperado**: `&Option<Box<TreeNode>>`

---

## TC-137: Print `node` em `height`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break height
rde> continue
rde> print node
```
**Esperado**: `&Option<Box<TreeNode>>`

---

## TC-138: Print `value` em `inorder_traversal`
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break inorder_traversal
rde> continue
rde> print value
```
**Esperado**: `&Option<Box<TreeNode>>`

---

## TC-139: Print `node` em `rotate_left`
```bash
rde> break rotate_left
rde> continue
rde> print node
```
**Esperado**: `&mut Option<Box<TreeNode>>`

---

## TC-140: Print `node` em `rotate_right`
```bash
rde> break rotate_right
rde> continue
rde> print node
```
**Esperado**: `&mut Option<Box<TreeNode>>`

---

## TC-141: Print estrutura completa `root` (3 níveis)
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> print *root
```
**Esperado**: Mostra TreeNode com left/right

---

## TC-142: Print `root.left.value`
```bash
rde> break insert
rde> continue
rde> print root.left.value
```
**Esperado**: `i32` ou None

---

## TC-143: Print `root.right.value`
```bash
rde> break insert
rde> continue
rde> print root.right.value
```
**Esperado**: `i32` ou None

---

## TC-144: Print `root.left.left.value`
```bash
rde> break insert
rde> continue
rde> print root.left.left.value
```
**Esperado**: `i32` ou None

---

## TC-145: Print em recurso profundo (nível 5)
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> step
rde> step
rde> step
rde> step
rde> step
rde> print value
```
**Esperado**: `i32` no nível 5

---

## TC-146: Print em recurso profundo (nível 10)
```bash
rde> break insert
rde> continue
rde> step (10x)
rde> print value
```
**Esperado**: `i32` no nível 10

---

## TC-147: Print tipo de `value`
```bash
rde> break insert
rde> continue
rde> print type(value)
```
**Esperado**: `i32`

---

## TC-148: Print tipo de `root`
```bash
rde> break insert
rde> continue
rde> print type(root)
```
**Esperado**: `Option<Box<TreeNode>>`

---

## TC-149: Print tipo de `root.left`
```bash
rde> break insert
rde> continue
rde> print type(root.left)
```
**Esperado**: `Option<Box<TreeNode>>`

---

## TC-150: Print valor nulo (`None`)
```bash
rde> break insert
rde> continue
rde> print root.left
```
**Esperado**: `None` ou `Option::None`

---

## TC-151: Print após modificação de valor
```bash
rde> break insert
rde> continue
rde> print value
rde> continue
rde> break insert
rde> continue
rde> print value
```
**Esperado**: Valor diferente

---

## TC-152: Print variável inexistente
```bash
rde> break insert
rde> continue
rde> print nonexistent
```
**Esperado**: Erro "não encontrada"

---

## TC-153: Print em módulo/ponto quebrado
```bash
rde> break insert
rde> continue
rde> print .
```
**Esperado**: Erro sintaxe

---

## TC-154: Print em múltiplos níveis de aninhamento
```bash
rde> break insert
rde> continue
rde> print root.left.right.left.value
```
**Esperado**: Navega aninhamento

---

## TC-155: Print após step em `insert`
```bash
rde> break insert
rde> continue
rde> step
rde> print value
```
**Esperado**: Valor correto no novo frame

---

## TC-156: Print variável não inicializada
**Nota**: Rust não permite, depurador mostra `<optimized>` ou similar

---

## TC-157: Print `size` em BinaryTree
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break main
rde> continue
rde> print tree.size
```
**Esperado**: `usize`

---

## TC-158: Print `tree.root` completo
```bash
rde> break main
rde> continue
rde> print tree.root
```
**Esperado**: `Option<Box<TreeNode>>`

---

## TC-159: Print após continue múltiplo
```bash
rde> break insert
rde> continue
rde> print value
rde> continue
rde> print value
rde> continue
rde> print value
```
**Esperado**: Valores diferentes

---

## TC-160: Print em demo diferente
```bash
rde-cli target/release/rust_app_example.exe --demo search-miss
rde> break search
rde> continue
rde> print value
```
**Esperado**: `i32`
