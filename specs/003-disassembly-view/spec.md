# Feature Specification: Disassembly View

**Feature Branch**: `003-disassembly-view`

**Created**: 2026-05-28

**Status**: Draft

**Input**: User description: "Visualização de código assembly com destaque de breakpoints. Disassembly em endereços específicos, sincronia com RIP."

## Clarifications

### Session 2026-05-28

- **Q**: Qual deve ser a estratégia de limite de memória lida para disassembly?  
  **→ A**: Heurística: `min(64 × count, 4KB)` — escala com o número de instruções solicitado, com teto absoluto de 4096 bytes para prevenir leituras massivas acidentais.
- **Q**: Como o engine deve lidar com símbolos ambíguos ao usar `disassemble <symbol>`?  
  **→ A**: Erro imediato listando todos os endereços encontrados — o usuário deve fornecer um endereço explícito (Option A).
- **Q**: Como o output deve indicar que uma instrução está "shadowed" por um breakpoint INT3?  
  **→ A**: Mostrar `int3` seguido de comentário com a instrução original (ex: `int3  ; was: mov rax, rbx`) — preserva o estado real da memória e dá contexto ao usuário (Option A).
- **Q**: Quantos bytes hex devem ser exibidos no output para cada instrução?  
  **→ A**: Máximo 8 bytes hex; instruções mais longas são truncadas com `..` (Option B).
- **Q**: O path de disassembly deve exigir `tracing` instrumentado em quais pontos?  
  **→ A**: Em todos os pontos principais: início do comando, leitura de memória, chamada ao Capstone, formatação do output, e erros (Option B).

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Disassemble at RIP on Breakpoint Hit (Priority: P1)

Como engenheiro de debugging, quando um breakpoint é atingido, quero ver as instruções de assembly ao redor do RIP (instruction pointer) do thread selecionado, para entender exatamente onde a execução parou no nível de máquina.

**Why this priority**: Esta é a funcionalidade core do feature. Sem disassembly contextual no ponto de parada, o valor do view é zero. É o primeiro passo para qualquer análise de baixo nível.

**Independent Test**: Pode ser testada independentemente executando um debuggee simples, setando um breakpoint, e verificando se o comando `disassemble` (ou `disas`) mostra as instruções ao redor do RIP com a instrução atual claramente marcada.

**Acceptance Scenarios**:

1. **Given** um processo debuggee parado em um breakpoint, **When** o usuário executa `disassemble`, **Then** o engine retorna ~10 instruções de assembly (5 antes e 5 depois do RIP), com a instrução no RIP marcada com `=>`.
2. **Given** um processo debuggee parado, **When** o usuário executa `disassemble` sem thread selecionada, **Then** o engine usa o thread que reportou o último evento de parada (ou thread 0 como fallback).
3. **Given** um processo debuggee em execução (não parado), **When** o usuário tenta `disassemble`, **Then** o engine retorna um erro claro: "Target is running. Pause execution first."

---

### User Story 2 — Disassemble at Arbitrary Address (Priority: P2)

Como engenheiro de debugging, quero poder solicitar disassembly em um endereço específico (ex: `disassemble 0x7ff6_1234_0000`), para inspecionar código em módulos carregados, stubs, ou regiões de memória conhecidas.

**Why this priority**: Permite navegação livre pelo espaço de código do processo, não apenas no ponto de parada. Essencial para análise de call targets, trampolines, e VTables.

**Independent Test**: Pode ser testada independentemente carregando um debuggee, obtendo o base address de um módulo via `modules`, e executando `disassemble <base>` para ver o entry point.

**Acceptance Scenarios**:

1. **Given** um processo debuggee parado, **When** o usuário executa `disassemble 0x140001000`, **Then** o engine lê memória nesse endereço, disassembla com Capstone, e retorna as instruções a partir desse ponto.
2. **Given** um processo debuggee parado, **When** o usuário executa `disassemble main` (símbolo conhecido), **Then** o engine resolve o símbolo para endereço via `rde-symbols` e disassembla a partir desse endereço.
3. **Given** um endereço inválido ou não-mapeado, **When** o usuário solicita disassembly, **Then** o engine retorna um erro claro: "Cannot read memory at 0x...: [reason]".

---

### User Story 3 — Breakpoint Highlighting in Disassembly (Priority: P2)

Como engenheiro de debugging, quero que instruções onde breakpoints estão ativos sejam marcadas visualmente no output de disassembly (ex: `b+` ou cor diferente), para não precisar correlacionar manualmente endereços de breakpoint com código assembly.

**Why this priority**: Reduz drasticamente a carga cognitiva ao debugar com múltiplos breakpoints. É uma feature de UX que eleva o valor do disassembly view de "nice to have" para ferramenta produtiva.

**Independent Test**: Pode ser testada independentemente setando breakpoints em endereços conhecidos, executando `disassemble` no range que cobre esses endereços, e verificando que as linhas correspondentes têm marcador de breakpoint.

**Acceptance Scenarios**:

1. **Given** um breakpoint ativo no endereço `0x140001020`, **When** o usuário executa `disassemble` no range que inclui esse endereço, **Then** a linha da instrução em `0x140001020` é prefixada com `b+` (indicando breakpoint ativo).
2. **Given** um breakpoint que foi removido, **When** o usuário executa `disassemble` no mesmo range, **Then** o marcador `b+` não aparece mais naquela linha.
3. **Given** múltiplos breakpoints no mesmo range de disassembly, **When** o usuário executa `disassemble`, **Then** todas as instruções afetadas exibem `b+`.

---

### User Story 4 — Sincronia Automática com RIP (Priority: P3)

Como engenheiro de debugging, após executar `step`, `next`, ou `continue` que resulta em parada, quero que o disassembly view seja reemitido automaticamente centrado no novo RIP, mantendo contexto contínuo sem precisar digitar `disassemble` repetidamente.

**Why this priority**: Feature de conveniência que melhora o fluxo de debugging interativo. Não é essencial para MVP mas eleva a experiência do REPL.

**Independent Test**: Pode ser testada independentemente habilitando auto-disassembly, dando `step`, e verificando que o output do disassembly aparece automaticamente após cada parada.

**Acceptance Scenarios**:

1. **Given** o modo `auto-disassemble` habilitado (`set auto-disassemble on`), **When** o debuggee para após um `step`, **Then** o REPL exibe automaticamente as instruções ao redor do novo RIP antes do prompt.
2. **Given** o modo `auto-disassemble` desabilitado (default: `off`), **When** o debuggee para após um `step`, **Then** o REPL NÃO exibe disassembly automaticamente.
3. **Given** `auto-disassemble on`, **When** ocorre um breakpoint hit, **Then** o disassembly ao redor do RIP do thread que bateu o breakpoint é exibido automaticamente.

---

### Edge Cases

- **Memória não executável**: O que acontece quando o usuário solicita disassembly em uma região de dados (heap, stack, .rdata)? Capstone tentará decodificar mesmo assim — o output será "garbage" bytes interpretados como instruções. Não é erro do engine, mas devemos documentar que disassembly em regiões não-.text é "best effort".
- **Memória parcialmente legível / leitura com limite**: `ReadProcessMemory` pode falhar no meio de uma instrução multi-byte, ou o buffer heurístico (`min(64 × count, 4KB)`) pode truncar o último opcode. Capstone para prematuramente nesses casos. O engine deve retornar o que conseguiu disassemblar + nota de "truncated".
- **Endereço nulo ou não alinhado**: Capstone lida com endereços arbitrários; não exigimos alinhamento. Endereço nulo deve retornar erro de leitura de memória.
- **Nenhum processo anexado**: `disassemble` sem sessão ativa deve retornar "No active debug session."
- **Símbolo não resolvido ou ambíguo**: Se o usuário passar um nome de símbolo que não existe, o engine retorna "Symbol 'xxx' not found". Se houver múltiplos matches, retorna "Ambiguous symbol 'xxx': found at 0x..., 0x..., 0x...".
- **Breakpoint na metadata da instrução**: Se um breakpoint INT3 (0xCC) foi escrito no início da instrução, o byte lido será 0xCC. O disassembly mostrará `int3` em vez da instrução original. O engine DEVE acrescentar um comentário indicando a instrução original (ex: `int3  ; was: mov rax, rbx`), obtendo o byte original do breakpoint manager.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O engine DEVE suportar um comando `disassemble` (alias: `disas`) que produz disassembly de código x86-64 usando Capstone Engine.
- **FR-002**: O comando `disassemble` sem argumentos DEVE disassemblar a partir do RIP do thread selecionado (ou thread do evento de parada).
- **FR-003**: O comando `disassemble <address>` DEVE aceitar endereços hexadecimais (prefixo `0x` opcional) e disassemblar a partir desse ponto.
- **FR-004**: O comando `disassemble <symbol>` DEVE resolver o nome do símbolo para endereço via `rde-symbols` e disassemblar a partir desse endereço.
- **FR-005**: O output DEVE incluir: endereço da instrução, bytes brutos (hex, máximo 8 bytes, truncado com `..` se maior), mnemônico, operandos, e marcadores `=>` (RIP) e `b+` (breakpoint).
- **FR-006**: O range padrão DEVE ser ~10 instruções (configurável via `set disassembly-count N`).
- **FR-007**: O engine DEVE ler memória do processo debuggee via `ReadProcessMemory` para alimentar o disassembler.
- **FR-008**: O engine DEVE marcar instruções que contêm breakpoints ativos com `b+` no output.
- **FR-009**: O REPL DEVE suportar `set auto-disassemble on|off` para controlar exibição automática após paradas.
- **FR-010**: Quando `auto-disassemble` está `on`, o REPL DEVE exibir disassembly ao redor do RIP automaticamente após `BreakpointHit`, `SingleStep`, e `Exception`.
- **FR-011**: O engine DEVE retornar erro claro se o target estiver em execução quando `disassemble` for solicitado.
- **FR-012**: O parser do REPL DEVE aceitar `disas`, `disassemble`, `disas <addr>`, `disassemble <addr>`, `disas <symbol>`.
- **FR-013**: O engine DEVE limitar a leitura de memória para disassembly a `min(64 × count, 4096)` bytes, onde `count` é o número de instruções solicitado. Nunca deve ler mais de 4KB em uma única chamada de disassembly.
- **FR-014**: Todo o path de disassembly (comando, leitura de memória, chamada ao Capstone, formatação, erros) DEVE emitir eventos `tracing` estruturados para satisfazer o Observability gate da Constituição.

### Key Entities

- **DisassemblyLine**: Representa uma linha de output de disassembly.
  - `address: u64` — endereço virtual da instrução
  - `bytes: Vec<u8>` — bytes brutos da instrução
  - `mnemonic: String` — mnemônico (ex: `mov`, `call`)
  - `operands: String` — operandos (ex: `rax, rbx`)
  - `is_current: bool` — true se address == RIP
  - `has_breakpoint: bool` — true se há breakpoint ativo neste endereço
- **DisassemblyView**: Coleção de `DisassemblyLine` com contexto.
  - `lines: Vec<DisassemblyLine>`
  - `base_address: u64` — endereço de início
  - `rip: u64` — RIP usado para centragem (se aplicável)
- **DisassemblyConfig**: Configuração do comportamento de disassembly.
  - `count: usize` — número de instruções (default: 10)
  - `auto_show: bool` — auto-disassemble após paradas (default: false)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Usuário consegue ver disassembly correto ao redor do RIP em <1s após comando `disassemble` em um processo de tamanho médio.
- **SC-002**: 100% dos breakpoints ativos no range visualizado são marcados com `b+` no output.
- **SC-003**: Comando `disassemble <symbol>` resolve símbolos em <500ms para PDBs carregados.
- **SC-004**: Output de disassembly é legível e formatado em colunas alinhadas (address | bytes | mnemonic | operands | markers).
- **SC-005**: Auto-disassemble respeita a flag `on|off` em 100% dos eventos de parada.

## Assumptions

- Capstone Engine (`capstone` crate 0.12) já está disponível no workspace e aprovado pela Constituição.
- `rde-symbols` possui API de resolução de símbolo por nome (ou terá até a implementação deste feature).
- O engine já suporta leitura de memória (`ReadProcessMemory`) e registradores (`GetThreadContext`).
- O breakpoint manager (`rde-breakpoint`) expõe uma API para listar endereços de breakpoints ativos.
- O formato de output é texto puro (não TUI/egui), adequado para REPL. TUI será pós-MVP com `ratatui`.
- Apenas arquitetura x86-64 é suportada (alinhado com Windows-Native Purity da Constituição).
