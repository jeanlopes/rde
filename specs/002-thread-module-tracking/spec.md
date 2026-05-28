# Feature Specification: Thread and Module Tracking

**Feature Branch**: `002-thread-module-tracking`

**Created**: 2026-05-28

**Status**: Draft

**Input**: User description: "Rastreamento de threads e módulos (DLLs) dinamicamente. Listar/trocar threads, listar módulos carregados."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Visualizar e Selecionar Threads (Priority: P1)

O desenvolvedor precisa ver todas as threads do programa alvo em execução e selecionar uma thread
específica para inspeção de registradores, stack trace e single-step.

**Why this priority**: Programas reais usam múltiplas threads. Sem saber quais threads existem e
qual está sendo inspecionada, o debugger opera no escuro. A thread selecionada define o contexto
para todos os comandos de inspeção de estado.

**Independent Test**: Pode ser testado independentemente lançando um programa com múltiplas
threads, executando o comando de listagem, e verificando que todas as threads aparecem com seus
identificadores e estados. A seleção de uma thread deve refletir nos comandos subsequentes de
`regs` e `bt`.

**Acceptance Scenarios**:

1. **Given** que o usuário lançou um programa para debug, **When** ele executa o comando de
   listar threads, **Then** o sistema deve exibir todas as threads do programa alvo com seus
   identificadores e estados atuais (rodando, suspensa, encerrada).
2. **Given** que múltiplas threads estão em execução, **When** o usuário seleciona uma thread
   específica, **Then** os comandos de inspeção de registradores (`regs`), stack trace (`bt`) e
   passo a passo (`step`) devem operar no contexto dessa thread selecionada.
3. **Given** que uma nova thread é criada durante a execução do programa alvo, **When** o debugger
   detecta o evento de criação, **Then** a nova thread deve aparecer automaticamente na lista de
   threads sem reiniciar a sessão.
4. **Given** que uma thread é encerrada durante a execução, **When** o debugger detecta o evento
   de saída, **Then** a thread deve ser marcada como encerrada na lista ou removida,
   e a thread selecionada deve ser ajustada se necessário.

---

### User Story 2 - Rastrear Módulos Carregados (Priority: P2)

O desenvolvedor precisa ver quais DLLs e executáveis estão carregados no espaço de endereço do
programa alvo, incluindo seus endereços base, para entender o mapa de memória e resolver símbolos.

**Why this priority**: A resolução de símbolos depende de saber quais módulos estão carregados e
onde. Sem o rastreamento de módulos, o stack trace mostra endereços brutos em vez de nomes de
função para código em DLLs do sistema ou bibliotecas dinâmicas carregadas em runtime.

**Independent Test**: Pode ser testado independentemente lançando um programa que carrega DLLs
(por exemplo, via `LoadLibrary`), executando o comando de listar módulos, e verificando que as
DLLs aparecem com nome e endereço base corretos.

**Acceptance Scenarios**:

1. **Given** que o usuário lançou um programa para debug, **When** ele executa o comando de listar
   módulos, **Then** o sistema deve exibir todas as DLLs e o executável principal com seus nomes,
   endereços base e tamanhos.
2. **Given** que o programa alvo carrega uma nova DLL dinamicamente durante a execução, **When**
   o debugger detecta o evento de carga, **Then** a nova DLL deve aparecer automaticamente na
   lista de módulos com seu nome e endereço base.
3. **Given** que o programa alvo descarrega uma DLL, **When** o debugger detecta o evento de
   descarga, **Then** a DLL deve ser removida da lista de módulos ativos.

---

### User Story 3 - Integração com Resolução de Símbolos (Priority: P3)

Quando uma DLL é carregada, o debugger deve notificar o mecanismo de resolução de símbolos para
que funções dentro dessa DLL possam ser resolvidas em stack traces e breakpoints por nome.

**Why this priority**: Sem essa integração, o usuário só vê endereços brutos para código em DLLs
recém-carregadas. A integração automática torna o debugging produtivo sem exigir que o usuário
recarregue manualmente os símbolos.

**Independent Test**: Pode ser testado independentemente verificando que, após o carregamento de
uma DLL com símbolos PDB, o stack trace exibe nomes de função dessa DLL em vez de endereços
hexadecimais brutos.

**Acceptance Scenarios**:

1. **Given** que uma DLL com símbolos PDB é carregada pelo programa alvo, **When** o debugger
   processa o evento de carga, **Then** o mecanismo de símbolos deve ser notificado para carregar
   os símbolos dessa DLL.
2. **Given** que os símbolos de uma DLL foram carregados, **When** o usuário solicita um stack
   trace contendo frames dessa DLL, **Then** os nomes de função devem ser exibidos em vez de
   endereços brutos.

---

### Edge Cases

- **Thread principal encerrada antes do processo**: O sistema deve continuar rastreando as
  threads restantes e manter a sessão ativa até o evento de saída do processo.
- **Módulo carregado sem símbolos**: O sistema deve listar o módulo normalmente, mas marcar que
  símbolos não estão disponíveis. O stack trace deve mostrar endereços brutos para frames desse
  módulo.
- **Múltiplas threads criadas simultaneamente**: O sistema deve processar todos os eventos de
  criação de thread na ordem em que chegam, sem perder nenhuma thread do registro.
- **Descarga de DLL que ainda está em uso no stack trace**: O sistema deve remover a DLL da lista
  de módulos ativos, mas o stack trace histórico ainda pode referenciá-la. Isso é aceitável — o
  foco é no estado atual.
- **Tentativa de selecionar thread inexistente ou encerrada**: O sistema deve rejeitar a seleção
  e informar o usuário que a thread não existe ou já foi encerrada.
- **Nenhuma thread no processo**: O sistema deve exibir uma mensagem indicando que nenhuma thread
  está ativa (situação transitória logo após o lançamento).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema deve rastrear a criação de threads do programa alvo em tempo real,
  registrando o identificador da thread e seu estado inicial.
- **FR-002**: O sistema deve rastrear a saída (encerramento) de threads, atualizando o estado da
  thread no registro para refletir que foi encerrada.
- **FR-003**: O sistema deve manter um registro atualizado de todas as threads ativas do programa
  alvo, acessível ao usuário a qualquer momento durante a sessão de debug.
- **FR-004**: O sistema deve permitir que o usuário liste todas as threads, exibindo identificador,
  estado atual (rodando, suspensa, encerrada) e a thread atualmente selecionada.
- **FR-005**: O sistema deve permitir que o usuário selecione uma thread específica como thread
  ativa para inspeção.
- **FR-006**: O sistema deve usar a thread selecionada como contexto padrão para comandos de
  inspeção de estado (registradores, stack trace, single-step).
- **FR-007**: O sistema deve rastrear o carregamento de módulos (DLLs e executável principal),
  registrando nome, endereço base e tamanho.
- **FR-008**: O sistema deve rastrear a descarga de módulos, removendo-os do registro de módulos
  ativos.
- **FR-009**: O sistema deve permitir que o usuário liste todos os módulos carregados, exibindo
  nome, endereço base, tamanho e status de símbolos (carregados ou não).
- **FR-010**: O sistema deve notificar o mecanismo de resolução de símbolos quando um novo módulo
  é carregado, para que símbolos desse módulo possam ser resolvidos.
- **FR-011**: O sistema deve exibir mensagens no REPL quando threads são criadas ou encerradas,
  e quando módulos são carregados ou descarregados, mantendo o usuário informado sobre mudanças
  dinâmicas no programa alvo.
- **FR-012**: O sistema deve cachear handles de threads abertas para evitar abrir/fechar
  repetidamente handles ao interagir com a mesma thread.

### Key Entities *(include if feature involves data)*

- **Thread**: Representa uma thread do programa alvo. Atributos: identificador único (ThreadId),
  handle do sistema operacional (opcional, cacheado), estado atual (rodando, suspensa, encerrada),
  contexto de execução (registradores, quando pausada).
- **Módulo**: Representa um executável ou DLL carregado no espaço de endereço do programa alvo.
  Atributos: nome, endereço base, tamanho, caminho no sistema de arquivos (quando disponível),
  status de carregamento de símbolos.
- **Registro de Threads**: Coleção mantida pelo engine com todas as threads conhecidas do programa
  alvo. Atualizada dinamicamente por eventos do sistema operacional.
- **Registro de Módulos**: Coleção mantida pelo engine com todos os módulos carregados no programa
  alvo. Atualizada dinamicamente por eventos de carga/descarga de DLL.
- **Thread Selecionada**: Identificador da thread que serve como contexto padrão para comandos de
  inspeção de estado. Inicialmente é a thread principal do processo.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O usuário consegue listar todas as threads ativas do programa alvo em menos de 200ms
  após o comando, mesmo com 50+ threads em execução.
- **SC-002**: A thread selecionada pelo usuário é usada como contexto em 100% dos comandos de
  inspeção de estado (`regs`, `bt`, `step`) sem exigir especificação explícita da thread.
- **SC-003**: 100% das threads criadas e encerradas durante a execução do programa alvo são
  refletidas no registro de threads dentro de 100ms do evento do sistema operacional.
- **SC-004**: O usuário consegue listar todos os módulos carregados em menos de 300ms, incluindo
  o executável principal e todas as DLLs do sistema carregadas pelo programa alvo.
- **SC-005**: Após o carregamento de uma DLL com símbolos PDB, o stack trace resolve nomes de
  função dessa DLL em pelo menos 90% dos frames dentro de 5 segundos do carregamento.
- **SC-006**: O debugger não vaza handles de threads ao longo de uma sessão de 10 minutos com
  criação e destruição contínua de threads (validado via contagem de handles abertos/fechados).

## Assumptions

- O programa alvo é um executável Windows nativo (PE/COFF) que pode criar threads via API Win32
  (`CreateThread`, `NtCreateThread`, etc.).
- A resolução de símbolos usa a API `DbgHelp.dll` do Windows (`SymInitialize`, `SymLoadModuleEx`,
  `SymFromAddr`) e requer que arquivos PDB estejam disponíveis no mesmo diretório do executável
  ou em um caminho de símbolos configurado.
- O rastreamento de threads e módulos é local ao processo debugado — não há suporte para
  debugging remoto ou multi-processo no MVP.
- A contagem de threads ativas é limitada pelo que o sistema operacional permite para um processo
  de usuário (tipicamente milhares), e o registro deve escalar linearmente.
- A detecção de nomes de módulos a partir de eventos de carga de DLL pode exigir leitura de
  memória do processo alvo para resolver o caminho do arquivo quando o handle do arquivo não
  fornece informação direta.
