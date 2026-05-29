# TC-211 a TC-230: REPL Runtime (Atomic)

---

## TC-211: Comando `help`
```bash
rde-cli target/release/rust_app_example.exe
rde> help
```
**Esperado**: Lista de comandos

---

## TC-212: Comando vazio
```bash
rde> 
```
**Esperado**: Nada ou prompt

---

## TC-213: Comando inválido
```bash
rde> foobar
```
**Esperado**: Erro "comando não reconhecido"

---

## TC-214: Comando com argumentos extras
```bash
rde> break insert extra
```
**Esperado**: Erro ou ignora extra

---

## TC-215: Comando `quit`
```bash
rde> quit
```
**Esperado**: Sai do REPL

---

## TC-216: Comando `exit`
```bash
rde> exit
```
**Esperado**: Sai do REPL

---

## TC-217: Ctrl+C no REPL
```bash
rde-cli target/release/rust_app_example.exe
# pressione Ctrl+C
```
**Esperado**: Interrompe ou ignora

---

## TC-218: Ctrl+D no REPL
```bash
rde> # pressione Ctrl+D
```
**Esperado**: EOF, sai do REPL

---

## TC-219: Tab completion de comando
```bash
rde> bre<TAB>
```
**Esperado**: Completa para break

---

## TC-220: Tab completion de símbolo
```bash
rde> break in<TAB>
```
**Esperado**: Completa para insert

---

## TC-221: Histórico de comandos (up arrow)
```bash
rde> break main
rde> continue
# pressione UP
```
**Esperado**: Mostra "continue"

---

## TC-222: Histórico com DOWN arrow
```bash
rde> break main
rde> continue
rde> step
# UP, UP, DOWN
```
**Esperado**: Navega histórico

---

## TC-223: REPL com TUI ativo
**Nota**: Requer TUI

---

## TC-224: REPL em modo batch
**Nota**: Se suportado

---

## TC-225: REPL com saída redirecionada
```bash
rde-cli target/release/rust_app_example.exe > output.txt
```
**Esperado**: Funciona com pipe

---

## TC-226: REPL com input de arquivo
```bash
cat commands.txt | rde-cli target/release/rust_app_example.exe
```
**Esperado**: Executa comandos

---

## TC-227: REPL mantém estado entre comandos
```bash
rde> break main
rde> continue
rde> bt
```
**Esperado**: Estado persistente

---

## TC-228: REPL mantém breakpoints entre sessões
**Nota**: Se persistência implementada

---

## TC-229: REPL após detach
**Nota**: Se attach suportado

---

## TC-230: REPL após processo encerrado
```bash
rde> break main
rde> continue
rde> continue
rde> bt
```
**Esperado**: Erro "processo encerrado"
