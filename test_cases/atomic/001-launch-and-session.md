# TC-001 a TC-015: Launch & Session Management (Atomic)

Atomic test cases — each is independent and can be run in isolation.

---

## TC-001: Launch standalone do binário
```bash
rde-cli target/release/rust_app_example.exe
```
**Esperado**: `Processo iniciado: PID <n>` + prompt `rde>`

---

## TC-002: Launch standalone com argumentos
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
```
**Esperado**: `Processo iniciado: PID <n>` + prompt `rde>`

---

## TC-003: Launch via Cargo (default)
```bash
cd examples/rust_app_example
rde-cli cargo debug
```
**Esperado**: Target resolvido, build se stale, `Processo iniciado: PID <n>`

---

## TC-004: Launch via Cargo com profile release
```bash
cd examples/rust_app_example
rde-cli cargo debug --release
```
**Esperado**: Binário release carregado, `Processo iniciado: PID <n>`

---

## TC-005: Launch via Cargo com package/bin específicos
```bash
rde-cli cargo debug --package rust_app_example --bin rust_app_example
```
**Esperado**: Target correto resolvido

---

## TC-006: Launch via Cargo com features
```bash
rde-cli cargo debug --features "some-feature"
```
**Esperado**: Features compiladas e launch

---

## TC-007: Launch com TUI
```bash
rde-cli --tui target/release/rust_app_example.exe
```
**Esperado**: Interface TUI inicia

---

## TC-008: Attach a processo running
```bash
rde-cli
rde> attach <PID>
```
**Esperado**: `Processo anexado: PID <n>`

---

## TC-009: Sessão termina com `quit`
```bash
rde-cli target/release/rust_app_example.exe
rde> quit
```
**Esperado**: Debugger fecha sem crash

---

## TC-010: Sessão termina quando debuggee exits
```bash
rde-cli target/release/rust_app_example.exe
rde> continue
```
**Esperado**: `Processo encerrado com código: 0`

---

## TC-011: Detect stale artifact e rebuild
1. Modifique `examples/rust_app_example/src/main.rs`
2. `rde-cli cargo debug`
**Esperado**: Build automático disparado

---

## TC-012: Cargo build failure handling
1. Introduza erro de sintaxe em `main.rs`
2. `rde-cli cargo debug`
**Esperado**: Erro exibido, sessão não inicia

---

## TC-013: Launch de binário inexistente
```bash
rde-cli nonexistent_file.exe
```
**Esperado**: Erro claro, sem crash

---

## TC-014: Launch de arquivo não-executável
```bash
rde-cli README.md
```
**Esperado**: Erro claro, sem crash

---

## TC-015: Múltiplos launches na mesma sessão
```bash
rde-cli target/release/rust_app_example.exe
rde> continue
# Após encerrar
rde> rde-cli target/release/rust_app_example.exe
```
**Esperado**: Comportamento definido (rejeitar ou restart)
