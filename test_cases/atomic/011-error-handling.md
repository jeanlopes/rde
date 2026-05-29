# TC-286 a TC-290: Error Handling (Atomic)

---

## TC-286: Executable não encontrado
```bash
rde-cli nonexistent.exe
```
**Esperado**: Erro "arquivo não encontrado"

---

## TC-287: Breakpoint em símbolo inválido
```bash
rde-cli target/release/rust_app_example.exe
rde> break nonexistent
```
**Esperado**: Erro "símbolo não encontrado"

---

## TC-288: Comando em estado inválido
```bash
rde-cli target/release/rust_app_example.exe
rde> continue
# processo encerra
rde> continue
```
**Esperado**: Erro "processo não está em execução"

---

## TC-289: Acesso a memória inválida
```bash
rde> x/16wx 0x0
```
**Esperado**: Erro de acesso

---

## TC-290: Disassemble de endereço inválido
```bash
rde> disassemble 0x0
```
**Esperado**: Erro
