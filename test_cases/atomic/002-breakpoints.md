# TC-016 a TC-060: Breakpoint Management (Atomic)

---

## TC-016: Breakpoint por símbolo `main`
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
```
**Esperado**: `Breakpoint 1 definido em main`

---

## TC-017: Breakpoint por símbolo `insert`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
```
**Esperado**: `Breakpoint 1 definido em insert`

---

## TC-018: Breakpoint por símbolo `search`
```bash
rde-cli target/release/rust_app_example.exe --demo search-miss
rde> break search
```
**Esperado**: `Breakpoint 1 definido em search`

---

## TC-019: Breakpoint por símbolo `delete`
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
```
**Esperado**: `Breakpoint 1 definido em delete`

---

## TC-020: Breakpoint por símbolo `find_min`
```bash
rde-cli target/release/rust_app_example.exe
rde> break find_min
```
**Esperado**: `Breakpoint 1 definido em find_min`

---

## TC-021: Breakpoint por símbolo `find_max`
```bash
rde-cli target/release/rust_app_example.exe
rde> break find_max
```
**Esperado**: `Breakpoint 1 definido em find_max`

---

## TC-022: Breakpoint por símbolo `height`
```bash
rde-cli target/release/rust_app_example.exe
rde> break height
```
**Esperado**: `Breakpoint 1 definido em height`

---

## TC-023: Breakpoint por símbolo `size`
```bash
rde-cli target/release/rust_app_example.exe
rde> break size
```
**Esperado**: `Breakpoint 1 definido em size`

---

## TC-024: Breakpoint por símbolo `inorder_traversal`
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break inorder_traversal
```
**Esperado**: `Breakpoint 1 definido em inorder_traversal`

---

## TC-025: Breakpoint por símbolo `preorder_traversal`
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break preorder_traversal
```
**Esperado**: `Breakpoint 1 definido em preorder_traversal`

---

## TC-026: Breakpoint por símbolo `postorder_traversal`
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break postorder_traversal
```
**Esperado**: `Breakpoint 1 definido em postorder_traversal`

---

## TC-027: Breakpoint por símbolo `rotate_left`
```bash
rde-cli target/release/rust_app_example.exe
rde> break rotate_left
```
**Esperado**: `Breakpoint 1 definido em rotate_left`

---

## TC-028: Breakpoint por símbolo `rotate_right`
```bash
rde-cli target/release/rust_app_example.exe
rde> break rotate_right
```
**Esperado**: `Breakpoint 1 definido em rotate_right`

---

## TC-029: Breakpoint por símbolo `balance_factor`
```bash
rde-cli target/release/rust_app_example.exe
rde> break balance_factor
```
**Esperado**: `Breakpoint 1 definido em balance_factor` ou erro se não existir

---

## TC-030: Breakpoint por símbolo inexistente
```bash
rde-cli target/release/rust_app_example.exe
rde> break nonexistent_xyz
```
**Esperado**: Erro: símbolo não encontrado

---

## TC-031: Breakpoint por endereço hexadecimal
```bash
rde-cli target/release/rust_app_example.exe
rde> break 0x140001000
```
**Esperado**: `Breakpoint 1 definido em 0x140001000` ou erro

---

## TC-032: Múltiplos breakpoints simultâneos
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break main
rde> break insert
```
**Esperado**: Dois IDs diferentes retornados

---

## TC-033: Deleção de breakpoint por ID
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> delbreak 1
```
**Esperado**: `Breakpoint 1 removido.`

---

## TC-034: Deleção de breakpoint inexistente
```bash
rde-cli target/release/rust_app_example.exe
rde> delbreak 999
```
**Esperado**: Erro: breakpoint não encontrado

---

## TC-035: Deleção de todos os breakpoints
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> delbreak 1
rde> continue
```
**Esperado**: Processo encerra sem hits

---

## TC-036: Breakpoint em tempo de execução (dinâmico)
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break main
rde> continue
# Hit em main
rde> break insert
rde> continue
```
**Esperado**: Hit em insert (adicionado dinamicamente)

---

## TC-037: Remoção de breakpoint em tempo de execução
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
rde> delbreak 1
rde> continue
```
**Esperado**: Não re-hita

---

## TC-038: Breakpoint em recursão profunda (`insert`)
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit 1
rde> continue
# Hit 2 (recursão)
```
**Esperado**: Cada chamada recursiva dispara hit

---

## TC-039: Breakpoint em função chamada múltiplas vezes
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit 1
rde> continue
# Hit 2
rde> continue
# Hit 3
```
**Esperado**: Múltiplos hits com mesmo ID

---

## TC-040: Breakpoint hit mostra thread ID
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
```
**Esperado**: `[Breakpoint 1] Hit em 0x<addr> — Thread <tid>`

---

## TC-041: Breakpoint hit mostra endereço
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
```
**Esperado**: Endereço hex visível

---

## TC-042: Breakpoint na demo `insert-sequence`
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
```
**Esperado**: Hit em insert

---

## TC-043: Breakpoint na demo `delete-rebalance`
```bash
rde-cli target/release/rust_app_example.exe --demo delete-rebalance
rde> break delete
rde> continue
```
**Esperado**: Hit em delete

---

## TC-044: Breakpoint na demo `search-miss`
```bash
rde-cli target/release/rust_app_example.exe --demo search-miss
rde> break search
rde> continue
```
**Esperado**: Hit em search

---

## TC-045: Breakpoint na demo `full-traversal`
```bash
rde-cli target/release/rust_app_example.exe --demo full-traversal
rde> break inorder_traversal
rde> continue
```
**Esperado**: Hit em inorder_traversal

---

## TC-046: Breakpoint em `println!` macro
```bash
rde-cli target/release/rust_app_example.exe
rde> break std::io::_print
rde> continue
```
**Esperado**: Hit ou erro claro

---

## TC-047: Breakpoint em `Drop` de `Box<Node>`
```bash
rde-cli target/release/rust_app_example.exe
rde> break drop
rde> continue
```
**Esperado**: Hit ou erro claro

---

## TC-048: Breakpoint em função auxiliar privada
```bash
rde-cli target/release/rust_app_example.exe
rde> break update_height
rde> continue
```
**Esperado**: Hit se visível, erro se não

---

## TC-049: Breakpoint duplicado no mesmo símbolo
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> break main
```
**Esperado**: Dois IDs diferentes

---

## TC-050: Breakpoint com símbolo parcial
```bash
rde-cli target/release/rust_app_example.exe
rde> break ins
```
**Esperado**: Match parcial ou erro

---

## TC-051: Breakpoint em `main` com argumentos
```bash
rde-cli target/release/rust_app_example.exe --demo insert
rde> break main
rde> continue
```
**Esperado**: Hit em main

---

## TC-052: Breakpoint em `new`
```bash
rde-cli target/release/rust_app_example.exe
rde> break new
rde> continue
```
**Esperado**: `Breakpoint 1 definido em new`

---

## TC-053: Breakpoint em `clear`
```bash
rde-cli target/release/rust_app_example.exe
rde> break clear
rde> continue
```
**Esperado**: `Breakpoint 1 definido em clear`

---

## TC-054: Breakpoint em `is_empty`
```bash
rde-cli target/release/rust_app_example.exe
rde> break is_empty
rde> continue
```
**Esperado**: `Breakpoint 1 definido em is_empty`

---

## TC-055: Breakpoint em `PartialEq`
```bash
rde-cli target/release/rust_app_example.exe
rde> break eq
rde> continue
```
**Esperado**: Hit ou erro

---

## TC-056: System initial breakpoint
```bash
rde-cli target/release/rust_app_example.exe
```
**Esperado**: Processo pausado no system breakpoint, `continue` funciona

---

## TC-057: Breakpoint hit com auto-disassembly on
```bash
rde-cli target/release/rust_app_example.exe
rde> set auto-disassemble on
rde> break main
rde> continue
```
**Esperado**: Disassembly exibido automaticamente

---

## TC-058: Breakpoint hit com auto-disassembly off
```bash
rde-cli target/release/rust_app_example.exe
rde> set auto-disassemble off
rde> break main
rde> continue
```
**Esperado**: Sem disassembly automático

---

## TC-059: Continue após breakpoint hit
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
rde> continue
```
**Esperado**: Próximo hit ou fim

---

## TC-060: Step após breakpoint hit
```bash
rde-cli target/release/rust_app_example.exe
rde> break main
rde> continue
# Hit
rde> step
```
**Esperado**: Single step executado
