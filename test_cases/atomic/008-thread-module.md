# TC-231 a TC-250: Thread & Module (Atomic)

---

## TC-231: Listar threads
```bash
rde-cli target/release/rust_app_example.exe
rde> threads
```
**Esperado**: Lista com main thread

---

## TC-232: Selecionar thread
```bash
rde> threads
rde> thread 0
```
**Esperado**: Thread selecionada

---

## TC-233: Listar módulos
```bash
rde> modules
```
**Esperado**: Lista de DLLs

---

## TC-234: Info de módulo principal
```bash
rde> modules
rde> module rust_app_example.exe
```
**Esperado**: Endereço base, tamanho

---

## TC-235: Info de módulo ntdll
```bash
rde> module ntdll.dll
```
**Esperado**: Endereço base

---

## TC-236: Info de módulo kernel32
```bash
rde> module kernel32.dll
```
**Esperado**: Endereço base

---

## TC-237: Info de módulo não carregado
```bash
rde> module nonexistent.dll
```
**Esperado**: Erro

---

## TC-238: Threads durante execução
```bash
rde> break main
rde> continue
rde> threads
```
**Esperado**: Lista threads suspensas

---

## TC-239: Módulos durante execução
```bash
rde> break main
rde> continue
rde> modules
```
**Esperado**: Lista módulos carregados

---

## TC-240: Módulos após carregamento de DLL
**Nota**: Depende do que o programa carrega

---

## TC-241: Tasks (se disponível)
```bash
rde> tasks
```
**Esperado**: Lista de tarefas

---

## TC-242: Info de thread específica
```bash
rde> threads
rde> thread 0 info
```
**Esperado**: Detalhes da thread

---

## TC-243: Módulo com símbolos
```bash
rde> module rust_app_example.exe symbols
```
**Esperado**: Lista de símbolos

---

## TC-244: Módulo sem símbolos (system DLL)
```bash
rde> module ntdll.dll symbols
```
**Esperado**: Sem símbolos ou lista limitada

---

## TC-245: Thread em demo single-threaded
```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
rde> threads
```
**Esperado**: 1 thread

---

## TC-246: Módulos em demo
```bash
rde> break insert
rde> continue
rde> modules
```
**Esperado**: Lista de módulos

---

## TC-247: Comando `info`
```bash
rde> info
```
**Esperado**: Info geral

---

## TC-248: Comando `info registers`
```bash
rde> break main
rde> continue
rde> info registers
```
**Esperado**: Registradores

---

## TC-249: Comando `info breakpoints`
```bash
rde> break main
rde> info breakpoints
```
**Esperado**: Lista de breakpoints

---

## TC-250: Comando `info locals`
```bash
rde> break insert
rde> continue
rde> info locals
```
**Esperado**: Variáveis locais
