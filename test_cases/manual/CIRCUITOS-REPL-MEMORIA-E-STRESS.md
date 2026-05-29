# Circuitos: REPL Runtime, Memory, Disassembly e Stress

Circuitos que testam **resiliência sob carga** e **estabilidade de comandos avançados** em sessões prolongadas.

---

## CIRCUITO-40: Maratona REPL — 50 Comandos em Sequência sem Quit

**Objetivo**: Verificar que o REPL não degrada após muitos comandos.
**Duração estimada**: 5 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
```

| Passo | Comando | Tipo |
|-------|---------|------|
| 1 | `print value` | Inspection |
| 2 | `regs` | Inspection |
| 3 | `bt` | Inspection |
| 4 | `vars` | Inspection |
| 5 | `set pretty-print off` | Config |
| 6 | `print root` | Inspection (raw) |
| 7 | `set pretty-print on` | Config |
| 8 | `print root` | Inspection (pretty) |
| 9 | `set print-depth 1` | Config |
| 10 | `print root` | Inspection |
| 11 | `set print-depth 5` | Config |
| 12 | `print root` | Inspection |
| 13 | `continue` | Execution |
| 14 | `print value` | Inspection (novo hit) |
| 15 | `x $rsp 32` | Memory |
| 16 | `x $rip 16` | Memory |
| 17 | `disassemble` | Disassembly |
| 18 | `set auto-disassemble on` | Config |
| 19 | `continue` | Execution (com auto-disassembly) |
| 20 | `set auto-disassemble off` | Config |
| 21 | `continue` | Execution |
| 22 | `print value` | Inspection |
| 23 | `threads` | Thread |
| 24 | `modules` | Module |
| 25 | `tasks` | Task |
| 26 | `continue` | Execution |
| 27 | `print value` | Inspection |
| 28 | `regs` | Inspection |
| 29 | `bt` | Inspection |
| 30 | `continue` | Execution |
| 31 | `print value` | Inspection |
| 32 | `vars` | Inspection |
| 33 | `continue` | Execution |
| 34 | `print value` | Inspection |
| 35 | `x $rsp 64` | Memory |
| 36 | `disassemble` | Disassembly |
| 37 | `continue` | Execution |
| 38 | `print value` | Inspection |
| 39 | `regs` | Inspection |
| 40 | `continue` | Execution |
| 41 | `print value` | Inspection |
| 42 | `bt` | Inspection |
| 43 | `continue` | Execution |
| 44 | `print value` | Inspection |
| 45 | `vars` | Inspection |
| 46 | `continue` | Execution |
| 47 | `print value` | Inspection |
| 48 | `regs` | Inspection |
| 49 | `continue` | Execution |
| 50 | `print value` | Inspection |
| 51 | `continue` | Processo encerra |

**Observação de resiliência**: Se em qualquer passo o REPL demorar > 3s para responder, há degradação de performance ou deadlock.

---

## CIRCUITO-41: Memory Examine em Todos os Endereços Importantes

**Objetivo**: Inspecionar código, stack e heap em uma única sessão.
**Duração estimada**: 3 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
# Hit
```

| Passo | Comando | O que inspeciona |
|-------|---------|-----------------|
| 1 | `x $rip 16` | Código da função atual |
| 2 | `x $rsp 32` | Stack (variáveis locais) |
| 3 | `x $rbp 16` | Stack frame anterior |
| 4 | `print root` | Obtém endereço do nó |
| 5 | `x <endereco_do_root> 24` | Heap (estrutura Node) |
| 6 | `print root.left` | Obtém endereço do filho |
| 7 | `x <endereco_left> 24` | Heap (outro nó) |
| 8 | `continue` | Próximo hit |
| 9 | `x $rip 16` | Código novamente (deve ser o mesmo endereço) |
| 10 | `x $rsp 32` | Stack (pode ter mudado) |
| 11 | `x <endereco_do_root> 24` | Heap (raiz, deve ser o mesmo) |
| 12 | `continue` | Próximo hit |
| 13 | Repita 8-12 | 5× seguidas |

**Observação de resiliência**: Se `x` em endereço de heap falhar após alguns hits (memória liberada ou corrompida), há bug no memory reader.

---

## CIRCUITO-42: Disassembly Automático e Manual Alternados

**Objetivo**: Testar toggles de auto-disassembly e comandos manuais.
**Duração estimada**: 3 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> set auto-disassemble on
rde> continue
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `continue` | Hit em insert + disassembly automático |
| 2 | `continue` | Próximo hit + disassembly automático |
| 3 | `set auto-disassemble off` | Desliga |
| 4 | `continue` | Hit SEM disassembly automático |
| 5 | `disassemble` | Manual — exibe assembly |
| 6 | `set disassembly-count 5` | Limita a 5 instruções |
| 7 | `disassemble` | Apenas 5 instruções |
| 8 | `set disassembly-count 50` | Aumenta para 50 |
| 9 | `disassemble` | 50 instruções |
| 10 | `continue` | Próximo hit |
| 11 | `disassemble` | Manual novamente |
| 12 | `set auto-disassemble on` | Liga de novo |
| 13 | `continue` | Hit + automático de volta |

---

## CIRCUITO-43: Threaded Debuggee — Inspeção Completa de Múltiplas Threads

**Objetivo**: Testar debugger com múltiplas threads ativas.
**Duração estimada**: 3 minutos

```bash
rde-cli target/release/rde-cli.exe crates/rde-cli/examples/threaded_debuggee.exe
```

| Passo | Comando | Validação |
|-------|---------|-----------|
| 1 | `threads` | Lista com >1 thread |
| 2 | `thread <tid_principal>` | Seleciona principal |
| 3 | `bt` | Stack trace da principal |
| 4 | `regs` | Registradores da principal |
| 5 | `thread <tid_worker_1>` | Seleciona worker 1 |
| 6 | `bt` | Stack trace do worker |
| 7 | `regs` | Registradores do worker |
| 8 | `thread <tid_worker_2>` | Seleciona worker 2 |
| 9 | `bt` | Stack trace |
| 10 | `thread <tid_principal>` | Volta para principal |
| 11 | `continue` | Processo continua |
| 12 | `threads` | Threads ainda listadas? Estados atualizados? |
| 13 | `modules` | Módulos carregados |
| 14 | `tasks` | `No Tokio runtime detected.` (debuggee não usa Tokio) |
| 15 | `continue` | Aguarde término das threads |
| 16 | `threads` | Threads encerradas? |

---

## CIRCUITO-44: Stress Test — 100 Hits sem Parar

**Objetivo**: Testar durabilidade pura do debugger.
**Duração estimada**: 10-15 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo stress-test
rde> break insert
rde> continue
```

**Instrução**: Execute `continue` 100× seguidas. Em cada hit (ou a cada 10 hits), execute um comando diferente:

| Hits | Comando a executar |
|------|-------------------|
| 1-10 | `continue` apenas |
| 11-20 | `print value` entre continues |
| 21-30 | `regs` entre continues |
| 31-40 | `bt` entre continues |
| 41-50 | `vars` entre continues |
| 51-60 | `x $rsp 16` entre continues |
| 61-70 | `disassemble` entre continues |
| 71-80 | `threads` entre continues |
| 81-90 | `modules` entre continues |
| 91-100 | `print root` entre continues |

**Observação de resiliência**:
- Conte o tempo entre `continue` e o próximo hit. Deve ser < 2s.
- Se o tempo crescer ao longo dos hits, há vazamento de performance.
- Se o debugger crashar antes dos 100, há bug de estabilidade.

---

## CIRCUITO-45: Sessão com Comandos Inválidos Intercalados

**Objetivo**: Verificar que comandos inválidos não corrompem o estado do debugger.
**Duração estimada**: 3 minutos

```bash
rde-cli target/release/rust_app_example.exe --demo insert-sequence
rde> break insert
rde> continue
```

| Passo | Comando | Esperado |
|-------|---------|----------|
| 1 | `continue` | Hit |
| 2 | `print value` | Valor correto |
| 3 | `foobar_invalid` | Erro |
| 4 | `print value` | Ainda funciona? Valor correto? |
| 5 | `delbreak 999` | Erro |
| 6 | `print value` | Ainda funciona? |
| 7 | `x 0x0 16` | Erro (endereço inválido) |
| 8 | `print value` | Ainda funciona? |
| 9 | `break nonexistent_xyz` | Erro |
| 10 | `continue` | Próximo hit — ainda funciona? |
| 11 | `print value` | Valor correto? |
| 12 | `attach 99999` | Erro |
| 13 | `print value` | Ainda funciona? |
| 14 | `set invalid-config abc` | Erro |
| 15 | `continue` | Próximo hit |
| 16 | `print value` | Valor correto após 5 erros? |

**Observação de resiliência**: Se qualquer comando válido falhar após um comando inválido, o parser/executor tem bug de estado.

---

## Checklist de Execução

- [ ] CIRCUITO-40: Maratona REPL — 50 Comandos em Sequência
- [ ] CIRCUITO-41: Memory Examine em Todos os Endereços Importantes
- [ ] CIRCUITO-42: Disassembly Automático e Manual Alternados
- [ ] CIRCUITO-43: Threaded Debuggee — Inspeção Completa
- [ ] CIRCUITO-44: Stress Test — 100 Hits sem Parar
- [ ] CIRCUITO-45: Sessão com Comandos Inválidos Intercalados
