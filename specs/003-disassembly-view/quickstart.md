# Quickstart: Disassembly View

**Feature**: 003-disassembly-view
**Prerequisites**: RDE compilado (`cargo build`), debuggee de exemplo disponível

## Comandos de Disassembly

### Visualizar assembly no RIP atual

Quando o debuggee está parado (ex: após um breakpoint):

```
(disassembly-view) disassemble
```

Output exemplo:
```
     0x00007ff6_1234_0010: 48 83 ec 28          sub    rsp, 0x28
     0x00007ff6_1234_0014: 48 8b 05 00 00 00 00 mov    rax, qword ptr [rip]
=>   0x00007ff6_1234_001b: cc                   int3   ; was: call 0x7ff6_1234_0050
     0x00007ff6_1234_001c: 48 83 c4 28          add    rsp, 0x28
```

- `=>` marca a instrução no RIP
- `b+` marca instrução com breakpoint ativo
- `int3 ; was: ...` indica instrução original shadowed pelo breakpoint

### Disassemblar em endereço específico

```
(disassembly-view) disassemble 0x7ff612340000
(disassembly-view) disas 0x140001000
```

### Disassemblar por nome de símbolo

```
(disassembly-view) disassemble main
(disassembly-view) disas my_module::do_work
```

Se o símbolo for ambíguo (existe em múltiplos módulos):
```
Error: Ambiguous symbol 'main': found at 0x140001000, 0x7ff812340000
```

## Configuração

### Alterar número de instruções exibidas

```
(disassembly-view) set disassembly-count 20
```

Padrão: `10`. Máximo recomendado: `100`.

### Habilitar auto-disassemble

```
(disassembly-view) set auto-disassemble on
```

Com `on`, o REPL exibe automaticamente as instruções ao redor do RIP após:
- Breakpoint hit
- Single step (`step`, `stepi`)
- Exception

Para desabilitar:
```
(disassembly-view) set auto-disassemble off
```

## Dicas

- Disassembly em regiões não-executáveis (heap, stack, .rdata) funciona, mas o output será "best effort" — Capstone decodifica qualquer sequência de bytes.
- Se o output terminar com `(truncated)`, o buffer de memória acabou antes de completar todas as instruções solicitadas. Isso é normal perto de boundaries de memória.
- Use `modules` para descobrir base addresses de DLLs, depois `disassemble <base>` para ver o entry point.
