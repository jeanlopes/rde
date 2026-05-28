# Quick Start — RDE

Guia passo a passo para sua primeira sessão de debug com o RDE no Windows.

---

## 1. Pré-requisitos

Você precisa de apenas duas coisas:

1. **Windows 10 ou 11** (x86-64)
2. **Rust** (stable toolchain)

Se não tiver o Rust instalado, abra um PowerShell e execute:

```powershell
winget install Rustlang.Rustup
```

Ou baixe em [rustup.rs](https://rustup.rs).

Verifique a instalação:

```powershell
rustc --version
cargo --version
```

---

## 2. Clone e Build

```powershell
git clone <repo-url> rde
cd rde
```

Build do workspace completo:

```powershell
cargo build --workspace
```

Isso compila todos os crates (`rde-core`, `rde-win32`, `rde-repl`, etc.) e o binário CLI
`rde-cli`.

---

## 3. Build do Programa de Teste

O RDE inclui um programa mínimo para testar o debugger:

```powershell
cargo build --example hello_debuggee
```

O executável será gerado em:

```
target\debug\examples\hello_debuggee.exe
```

---

## 4. Primeira Sessão de Debug

Inicie o RDE com o programa alvo:

```powershell
cargo run --bin rde-cli -- target\debug\examples\hello_debuggee.exe
```

Você verá o prompt do REPL:

```
rde> Processo iniciado: hello_debuggee.exe (PID: 12345)
rde>
```

---

## 5. Comandos Básicos

### Colocar um breakpoint

```
rde> break main
Breakpoint 1 definido em main (0x00007FF612345678)
```

Ou por endereço:

```
rde> break 0x00007FF612345678
```

### Continuar execução

```
rde> continue
Executando...
```

O programa roda até encontrar o breakpoint.

### Quando o breakpoint dispara

```
[Breakpoint 1] Hit em main (0x00007FF612345678) — Thread 12345
rde>
```

Agora o processo está parado e você pode inspecionar tudo.

### Registradores

```
rde> regs
RAX: 0000000000000000  RBX: 0000000000000000
RCX: 000001F3AABBCC00  RDX: 0000000000000000
RIP: 00007FF612345678  RSP: 0000005BABCDEF00
RFLAGS: 00000202
```

### Memória

```
rde> x 0x00007FF612345678 16
0x00007FF612345678:  CC 55 48 8B EC 48 83 EC 30 48 89 4D 10 48 89 55  |.UH..H..0H.M.H.U|
```

O `CC` no início é o `INT3` do breakpoint.

### Passo a passo

```
rde> step
```

Executa uma instrução e para.

### Stack trace

```
rde> bt
#0 main         em hello_debuggee.rs:4
#1 __tmainCRTStartup em ucrtbase.dll
#2 BaseThreadInitThunk em KERNEL32.DLL
```

### Remover breakpoint

```
rde> delbreak 1
Breakpoint 1 removido.
```

### Sair

```
rde> quit
Sessão encerrada. Processo finalizado.
```

---

## 6. Breakpoints em Runtime

Esse é o diferencial do RDE. Enquanto o programa está rodando, o REPL continua respondendo.

```
rde> continue
Executando...

rde> break alguma_funcao   <-- você digita isso ENQUANTO roda
Breakpoint 2 definido em alguma_funcao (0x00007FF612349000)
```

O RDE suspende a thread alvo, escreve o `INT3` (0xCC), e resume — tudo via channel, sem
bloquear o REPL.

---

## 7. Comandos Disponíveis

| Comando | Descrição |
|---|---|
| `break <addr/sym>` | Define breakpoint |
| `delbreak <id>` | Remove breakpoint |
| `continue` | Continua execução |
| `step` | Step into (uma instrução) |
| `regs` | Mostra registradores x64 |
| `x <addr> [n]` | Examina memória (hex + ASCII) |
| `set <reg> = <val>` | Altera registrador |
| `bt` | Stack trace |
| `threads` | Lista threads |
| `modules` | Lista DLLs carregadas |
| `disas <addr> [n]` | Disassembly com Capstone |
| `attach <pid>` | Anexa em processo existente |
| `quit` | Sai do debugger |

---

## 8. Troubleshooting

### "Acesso negado" ao anexar em processo

Execute o PowerShell ou terminal como Administrador. `DebugActiveProcess` exige privilégios
elevados para processos de outros usuários ou alguns processos protegidos.

### Símbolos não aparecem no stack trace

O RDE usa `DbgHelp.dll`. Certifique-se de que o PDB do executável está no mesmo diretório do
`.exe`. Para builds Rust, o PDB é gerado automaticamente em `target/debug/` ao lado do `.exe`.

### O REPL não responde durante `continue`

Isso não deve acontecer no RDE. Se acontecer, é um bug — o debug loop thread está fazendo I/O
bloqueante fora de `WaitForDebugEventEx`. Abra uma issue.

### Antivírus bloqueia

O RDE usa APIs legítimas de debugging do Windows (`ReadProcessMemory`, `WriteProcessMemory`).
Alguns antivírus podem sinalizar. Adicione o diretório do projeto às exclusões do antivírus ou
execute em um ambiente de desenvolvimento confiável.

---

## Próximos Passos

- Leia a constituição do projeto (`.specify/memory/constitution.md`)
- Explore o código em `crates/rde-win32/` para ver as chamadas Win32
- Adicione uma feature nova seguindo o fluxo `/speckit-specify` → `/speckit-implement`
