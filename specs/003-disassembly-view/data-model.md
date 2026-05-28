# Data Model: Disassembly View (Feature 003)

**Date**: 2026-05-28
**Source**: [spec.md](./spec.md)

## Entities

### DisassemblyLine

Representa uma única linha de output de disassembly.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `address` | `u64` | Endereço virtual da instrução no processo debuggee | Alinhado a byte (não exige alinhamento de instrução) |
| `bytes` | `Vec<u8>` | Bytes brutos da instrução decodificada | Comprimento real da instrução (1–15 bytes) |
| `mnemonic` | `String` | Mnemônico Intel (ex: `mov`, `call`, `int3`) | Nunca vazio para instrução válida |
| `operands` | `String` | Operandos formatados (ex: `rax, rbx`, `qword ptr [rsp+8]`) | Pode ser vazio (ex: `nop`, `int3`) |
| `is_current` | `bool` | `true` se esta instrução está no RIP | Apenas uma linha por view pode ser `true` (ou zero, se RIP fora do range) |
| `has_breakpoint` | `bool` | `true` se há breakpoint ativo neste endereço | Consultado via breakpoint manager no momento da renderização |
| `original_bytes` | `Option<Vec<u8>>` | Bytes originais antes do INT3, se shadowed | Populado apenas quando `has_breakpoint == true` e breakpoint é software (INT3) |

### DisassemblyView

Coleção de linhas com contexto de centragem.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `lines` | `Vec<DisassemblyLine>` | Linhas de disassembly ordenadas por endereço | Tamanho ≤ `count` solicitado (pode ser menor se memória truncada) |
| `base_address` | `u64` | Endereço de início do buffer de memória lido | Pode ser menor que o endereço da primeira instrução (centramos em RIP) |
| `rip` | `Option<u64>` | RIP usado para centragem, se aplicável | `None` quando disassembly é feito em endereço arbitrário sem thread context |
| `truncated` | `bool` | `true` se o buffer de memória foi truncado antes de atingir `count` instruções | Indica que `ReadProcessMemory` retornou menos bytes ou Capstone parou prematuramente |

### DisassemblyConfig

Configuração persistente da sessão REPL (não do engine).

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `count` | `usize` | `10` | Número de instruções a exibir por comando |
| `auto_show` | `bool` | `false` | Se `true`, exibe disassembly automaticamente após eventos de parada |

## Relationships

```
Session (rde-core)
  ├── threads: HashMap<u32, Thread>
  ├── modules: HashMap<u64, Module>
  ├── breakpoints: BreakpointManager  (via rde-breakpoint)
  └── disasm_config: DisassemblyConfig

EngineCommand::Disassemble
  ├── address: Option<u64>        -- None → usar RIP do thread selecionado
  ├── thread_id: Option<u32>      -- None → usar selected_thread ou thread do evento
  └── count: Option<usize>        -- None → usar disasm_config.count

EngineEvent::Output(String)       -- Texto formatado do DisassemblyView
```

## State Transitions

DisassemblyView é **efêmero** — criado no handler do comando, formatado para string, enviado como `EngineEvent::Output`, e descartado. Não há estado persistente além de `DisassemblyConfig`.

## Validation Rules

1. `count` deve ser ≥ 1 e ≤ 100 (limite arbitrário para prevenir output excessivo)
2. `address` + `buffer_size` não deve exceder espaço de endereço de 64 bits (overflow check)
3. Se `auto_show == true` e target está running, auto-disassemble é suprimido silenciosamente (não gera erro)
