# Research: Disassembly View (Feature 003)

**Date**: 2026-05-28
**Feature**: Disassembly View
**Researcher**: speckit-plan agent

## Decisions

### 1. Capstone Engine como disassembler

**Decision**: Usar o crate `capstone` 0.12 em modo `CS_ARCH_X86` + `CS_MODE_64`.

**Rationale**:
- Aprovado explicitamente pela Constituição (Technology Stack: "Capstone Engine via `capstone` crate (x86-64 mode)")
- Bindings Rust maduras e amplamente usadas (ex: `rbspy`, `frida-rs`)
- Stateless entre chamadas — não mantém referência ao buffer de entrada, facilitando lifetime management
- Suporta syntax Intel (padrão Windows) via `CS_OPT_SYNTAX_INTEL`

**Alternatives considered**:
- `yaxpeax-x86` — puramente Rust, mas menos maduro e sem syntax Intel nativa
- `iced-x86` — excelente, mas maior overhead de compilação; Capstone já aprovado
- LLVM disassembler via `llvm-sys` — viola Constituição (proíbe dependências LLVM/LLDB)

**Code sketch**:
```rust
use capstone::prelude::*;

let cs = Capstone::new()
    .x86()
    .mode(arch::x86::ArchMode::Mode64)
    .syntax(arch::x86::ArchSyntax::Intel)
    .build()?;

let insns = cs.disasm_all(code, addr)?;
for insn in insns.iter() {
    println!("0x{:x}: {} {}", insn.address(), insn.mnemonic().unwrap(), insn.op_str().unwrap());
}
```

### 2. Estratégia de leitura de memória

**Decision**: Buffer de `min(64 × count, 4096)` bytes, lido em uma única chamada `ReadProcessMemory`.

**Rationale**:
- Instruções x86-64 têm comprimento variável (1–15 bytes). Para garantir N instruções completas, precisamos de buffer suficientemente grande.
- 64 bytes por instrução é ~4× o máximo teórico (15 bytes), cobrindo margem segura.
- Teto de 4KB previne leituras acidentais massivas e alinha com page size (reduz chance de falha parcial no meio de uma página não-mapeada).
- Se `ReadProcessMemory` retornar menos bytes que solicitado (mas >0), Capstone disassembla o que conseguiu e para — comportamento aceitável.

**Alternatives considered**:
- Leitura iterativa byte-a-byte até obter N instruções — complexo, múltiplas syscalls, lento
- Buffer fixo de 1KB — pode truncar ranges com instruções longas (ex: `movabs` com prefixos)

### 3. Formato de output textual

**Decision**: Colunas fixas: `[markers] address | bytes (≤8) | mnemonic | operands | [comment]`

**Rationale**:
- REPL textual precisa de legibilidade em terminais de largura variável
- 8 bytes hex = 17 caracteres (`xx xx xx xx xx xx xx xx` ou com `..`)
- Marcadores prefixados: `=>` para RIP, `b+` para breakpoint (podem combinar: `=> b+`)
- Comentário apenas para instruções shadowed por breakpoint (`; was: ...`)

**Example output**:
```
=>   0x00007ff6_1234_0020: 48 89 5c 24 08       mov    qword ptr [rsp+8], rbx
     0x00007ff6_1234_0025: 48 89 74 24 10       mov    qword ptr [rsp+16], rsi
 b+  0x00007ff6_1234_002a: cc                   int3   ; was: call 0x7ff6_1234_0100
     0x00007ff6_1234_002b: 48 83 c4 28          add    rsp, 0x28
```

### 4. Símbolos ambíguos

**Decision**: Erro explícito listando todos os matches.

**Rationale**:
- `DbgHelp` (`SymFromAddr`/`SymEnumSymbols`) pode retornar múltiplos símbolos com mesmo nome em módulos diferentes
- Escolher silenciosamente o "primeiro" é não-determinístico (depende de ordem de carregamento)
- Prompt interativo bloquearia o event loop async — viola REPL-First

## Unknowns Resolvido

Nenhum unknown crítico remanescente. Tecnologias e padrões são conhecidos e aprovados.
