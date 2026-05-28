# Feature Specification: Rust Debugger Engine (RDE)

**Feature Branch**: `001-debugger-engine`

**Created**: 2026-05-28

**Status**: Draft

**Input**: User description: "Implementar Rust Debugger Engine (RDE) nativo para Windows com Win32 Debug API, REPL embutido, breakpoints em runtime, e arquitetura async message-passing. Abandonar LLDB completamente."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Controle de Execução e Breakpoints (Priority: P1)

O desenvolvedor precisa iniciar uma sessão de debug, controlar a execução do programa (play, pause,
step) e definir pontos de parada em tempo real, sem que a interface congele.

**Why this priority**: É o núcleo mínimo de qualquer debugger. Sem controle de execução e
breakpoints funcionais, não há produto viável.

**Independent Test**: Pode ser testado independentemente lançando um programa simples,
pausando em um breakpoint, e verificando que a execução para no ponto correto.

**Acceptance Scenarios**:

1. **Given** que o usuário lançou um programa para debug, **When** ele define um breakpoint em
   uma função e continua a execução, **Then** o programa deve parar exatamente nessa função.
2. **Given** que o programa está rodando, **When** o usuário adiciona um novo breakpoint via
   interface sem pausar a execução, **Then** o breakpoint deve ser ativado e o programa deve
   parar nele quando o fluxo de execução passar por aquele ponto.
3. **Given** que o programa está parado em um breakpoint, **When** o usuário executa o comando
   de passo a passo (step), **Then** o programa deve avançar exatamente uma instrução e parar
   novamente.

---

### User Story 2 - REPL Interativo Responsivo (Priority: P2)

O desenvolvedor interage com o debugger através de uma interface de linha de comando (REPL) que
permanece responsiva o tempo todo, mesmo enquanto o programa alvo está executando.

**Why this priority**: A experiência de debug é definida pela fluidez da interface. Se o usuário
não pode digitar comandos enquanto o programa roda, o fluxo de trabalho é interrompido.

**Independent Test**: Pode ser testado independentemente verificando que o REPL aceita e
processa comandos em menos de 100ms mesmo com o programa alvo em execução contínua.

**Acceptance Scenarios**:

1. **Given** que o programa alvo está em execução contínua, **When** o usuário digita um comando
   no REPL, **Then** o comando deve ser aceito e processado sem travar a interface.
2. **Given** que o usuário está em uma sessão de debug, **When** ele digita comandos básicos
   (break, continue, step, quit), **Then** cada comando deve produzir feedback imediato e
   compreensível.

---

### User Story 3 - Inspeção de Estado do Programa (Priority: P3)

O desenvolvedor inspeciona o estado interno do programa pausado: registradores da CPU, conteúdo
da memória, e pilha de chamadas com nomes de funções resolvidos.

**Why this priority**: Debugar sem ver o estado é como operar no escuro. A inspeção de estado
permite ao desenvolvedor entender o que o programa está fazendo e por que parou.

**Independent Test**: Pode ser testado independentemente pausando o programa em um ponto
conhecido e verificando que os registradores, memória e stack trace refletem o estado esperado.

**Acceptance Scenarios**:

1. **Given** que o programa está pausado em um breakpoint, **When** o usuário solicita os
   registradores da CPU, **Then** o sistema deve exibir todos os registradores x64 com seus
   valores atuais.
2. **Given** que o programa está pausado, **When** o usuário solicita o conteúdo da memória em
   um endereço específico, **Then** o sistema deve exibir os bytes em formato hexadecimal e
   ASCII.
3. **Given** que o programa está pausado, **When** o usuário solicita a pilha de chamadas,
   **Then** o sistema deve exibir a sequência de funções chamadas, incluindo nomes de funções
   Rust legíveis (demangled).

---

### Edge Cases

- **Programa termina inesperadamente**: O debugger deve detectar a saída do processo e informar
  o usuário no REPL, encerrando a sessão graciosamente.
- **Breakpoint em endereço inválido**: O sistema deve rejeitar a tentativa e informar o usuário
  que o endereço não é válido ou não é executável.
- **Múltiplas threads**: O debugger deve rastrear a criação e destruição de threads e permitir
  que o usuário inspecione o contexto de qualquer thread individualmente.
- **Programa alvo carrega DLLs dinamicamente**: O sistema deve atualizar a lista de módulos
  carregados e resolver símbolos das novas DLLs automaticamente.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema deve permitir que o usuário inicie (launch) um programa para debug a
  partir de um caminho no sistema de arquivos.
- **FR-002**: O sistema deve permitir que o usuário anexe (attach) a um processo já em execução
  identificado por seu PID.
- **FR-003**: O sistema deve permitir pausar a execução do programa alvo e continuar
  (continue/resume) a execução sob demanda do usuário.
- **FR-004**: O sistema deve permitir avançar a execução instrução por instrução (single-step)
  quando o programa está pausado.
- **FR-005**: O sistema deve permitir definir, remover e listar breakpoints em endereços de
  memória ou nomes de função.
- **FR-006**: O sistema deve permitir adicionar e remover breakpoints enquanto o programa alvo
  está em execução, sem congelar a interface de usuário.
- **FR-007**: O sistema deve fornecer uma interface de linha de comando interativa (REPL) que
  aceite comandos do usuário e forneça feedback em tempo real.
- **FR-008**: O sistema deve exibir o estado completo dos registradores da CPU quando o programa
  está pausado.
- **FR-009**: O sistema deve permitir ler o conteúdo da memória do programa alvo em endereços
  específicos, exibindo os dados em formato legível.
- **FR-010**: O sistema deve permitir escrever na memória do programa alvo em endereços
  específicos, com confirmação de segurança quando apropriado.
- **FR-011**: O sistema deve exibir a pilha de chamadas (stack trace) do programa pausado,
  incluindo endereços de retorno e nomes de funções quando disponíveis.
- **FR-012**: O sistema deve resolver símbolos de funções de programas Rust, convertendo nomes
  mangled para nomes legíveis.
- **FR-013**: O sistema deve rastrear threads do programa alvo, permitindo ao usuário listar
  threads e selecionar uma thread para inspeção.
- **FR-014**: O sistema deve rastrear módulos (DLLs) carregados pelo programa alvo e atualizar
  a resolução de símbolos conforme novos módulos são carregados.
- **FR-015**: O sistema deve exibir a desassembly (código assembly) de trechos de memória
  especificados pelo usuário.

### Key Entities

- **Sessão de Debug**: Representa uma sessão ativa de debugging. Atributos: programa alvo,
  estado atual (rodando/pausado), lista de breakpoints, threads rastreadas.
- **Breakpoint**: Representa um ponto de parada configurado pelo usuário. Atributos: endereço,
  estado (ativo/inativo), contador de acionamentos.
- **Thread**: Representa uma thread do programa alvo. Atributos: identificador, estado,
  contexto de execução (registradores).
- **Módulo**: Representa uma DLL ou executável carregado no espaço de endereço do programa alvo.
  Atributos: nome, endereço base, tamanho, informações de símbolo.
- **Comando REPL**: Representa uma instrução do usuário para o debugger. Atributos: tipo de
  comando, argumentos, resposta/resultado.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O usuário consegue iniciar uma sessão de debug e alcançar o primeiro breakpoint
  em menos de 5 segundos após o comando de lançamento.
- **SC-002**: A interface REPL responde a comandos em menos de 100ms em 99% das interações,
  mesmo quando o programa alvo está em execução contínua.
- **SC-003**: Breakpoints podem ser adicionados e removidos em runtime sem causar pausa ou
  travamento perceptível na interface (tempo de resposta < 50ms).
- **SC-004**: O stack trace exibe nomes de funções Rust legíveis em pelo menos 95% dos
  frames quando símbolos de debug do programa alvo estão disponíveis.
- **SC-005**: 100% dos comandos core (break, continue, step, registers, memory read, stack trace,
  quit) executam sem travar o debugger ou o programa alvo.
- **SC-006**: O debugger sustenta uma sessão de pelo menos 10 minutos de execução contínua com
  múltiplos breakpoints sendo acionados repetidamente sem vazamento de recursos ou degradação
  de performance.

## Assumptions

- O usuário executa o debugger em um ambiente Windows 10 ou 11 (x86-64) com privilégios
  adequados para depuração de processos.
- O programa alvo é um executável nativo Windows (PE/COFF) com símbolos PDB gerados durante a
  compilação.
- O usuário possui conhecimento básico de debugging e está familiarizado com conceitos como
  breakpoints, registradores e stack traces.
- Para o MVP, o foco é em programas Rust compilados para Windows; suporte a outras linguagens
  é desejável mas não obrigatório na primeira versão.
- A interface TUI gráfica (ratatui) está fora do escopo do MVP e será tratada em uma fase
  posterior.
