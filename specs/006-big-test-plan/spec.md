# Feature Specification: Big Test Plan — rde-cli with Binary Tree Debuggee

**Feature Branch**: `006-big-test-plan`

**Created**: 2026-05-28

**Status**: Draft

**Input**: User description: "O grande plano de teste. Analisar o aplicativo rust_app_example (árvore binária). Criar casos de teste usando o rde-cli com todas as features possíveis. Navegar por dentro do app de árvore binária nos casos de teste, usar cada função e depurar cada função da árvore. Use a demo. Faça Continue, Step In, Step Out, Step Over. Inspecione variáveis de cada contexto, adicione pequenos comandos em tempo de execução com o REPL, coloque breakpoints em tempo de execução, remova breakpoints, dê um continue. Teste tudo. Mais de 200 test cases diferentes. Todos usando o CLI."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Launch & Session Initialization (Priority: P1)

Como engenheiro de QA, quero iniciar sessões de debug da aplicação de árvore binária de múltiplas formas para garantir que o rde-cli consegue carregar qualquer tipo de alvo Rust.

**Why this priority**: Sem capacidade de launch confiável, nenhum outro teste pode ser executado.

**Independent Test**: Cada cenário de launch pode ser validado isoladamente verificando se o processo é iniciado, o REPL aparece e o evento `ProcessLaunched` é emitido.

**Acceptance Scenarios**:

1. **Given** o binário `rust_app_example.exe` compilado, **When** executo `rde-cli rust_app_example.exe`, **Then** o REPL inicia e o evento `ProcessLaunched` é recebido com PID válido.
2. **Given** o projeto Cargo da árvore binária, **When** executo `rde-cli cargo debug`, **Then** o binário é resolvido, build é disparado se necessário e o debug inicia.
3. **Given** o projeto Cargo, **When** executo `rde-cli cargo debug --release`, **Then** o binário de release é carregado corretamente.
4. **Given** o projeto Cargo em workspace, **When** executo `rde-cli cargo debug --package tree --bin rust_app_example`, **Then** o target correto é resolvido e debugado.
5. **Given** o binário com argumentos `--demo`, **When** executo `rde-cli rust_app_example.exe --demo`, **Then** os argumentos são repassados e a demo é executada sob debug.

---

### User Story 2 — Breakpoint Management (Priority: P1)

Como engenheiro de QA, quero criar, listar e remover breakpoints em todos os pontos críticos da árvore binária para controlar a execução e inspecionar estado.

**Why this priority**: Breakpoints são a base da depuração interativa; sem eles não é possível inspecionar estado em tempo de execução.

**Independent Test**: Cada operação de breakpoint pode ser testada isoladamente verificando confirmação no REPL e evento `BreakpointHit` correspondente.

**Acceptance Scenarios**:

1. **Given** sessão ativa, **When** executo `break main`, **Then** o breakpoint é definido e o ID é retornado.
2. **Given** sessão ativa, **When** executo `break 0x<endereço_hex>`, **Then** o breakpoint por endereço é definido.
3. **Given** breakpoint ativo, **When** executo `delbreak <id>`, **Then** o breakpoint é removido e não dispara mais.
4. **Given** múltiplos breakpoints, **When** executo `delbreak` em todos, **Then** a lista fica vazia e `continue` executa até o fim.
5. **Given** sessão em execução, **When** adiciono um breakpoint via REPL sem parar o processo (se suportado), **Then** o breakpoint é ativado dinamicamente.

---

### User Story 3 — Execution Control Flow (Priority: P1)

Como engenheiro de QA, quero controlar o fluxo de execução (continue, step into, step over, step out) em cada função da árvore binária para validar o comportamento do debugger em diferentes profundidades de call stack.

**Why this priority**: Controle de execução é essencial para navegar pelo código e validar a lógica da árvore binária passo a passo.

**Independent Test**: Cada comando de controle pode ser testado em uma função isolada da árvore verificando o endereço RIP e o evento correspondente (`BreakpointHit`, `SingleStep`).

**Acceptance Scenarios**:

1. **Given** parado em `main`, **When** executo `continue`, **Then** a execução prossegue até o próximo breakpoint ou término.
2. **Given** parado em `insert`, **When** executo `step` (step into), **Then** o debugger entra na próxima função chamada (ex: `Box::new` ou rotação).
3. **Given** dentro de `insert`, **When** executo step over em uma chamada recursiva, **Then** o debugger para após o retorno da chamada sem entrar.
4. **Given** dentro de uma função auxiliar profunda (ex: `rotate_left`), **When** executo step out, **Then** o debugger retorna até o chamador (`insert` ou `delete`).
5. **Given** múltiplos níveis de recursão em `insert`, **When** executo step into repetidamente, **Then** o debugger desce cada nível até atingir a folha.

---

### User Story 4 — Variable & State Inspection (Priority: P1)

Como engenheiro de QA, quero inspecionar variáveis locais, parâmetros, ponteiros e registros em cada função da árvore binária para validar a integridade dos dados em tempo de execução.

**Why this priority**: A principal função de um debugger é revelar estado interno; sem inspeção confiável não é possível depurar a lógica da árvore.

**Independent Test**: Cada comando de inspeção pode ser testado em um breakpoint isolado comparando o valor exibido com o estado esperado da árvore.

**Acceptance Scenarios**:

1. **Given** parado em `insert(value: 42)`, **When** executo `print value`, **Then** o valor 42 (ou pretty-print correspondente) é exibido.
2. **Given** parado em `insert` com um nó existente, **When** executo `print node`, **Then** a estrutura do nó (left, right, value) é exibida.
3. **Given** parado em qualquer função, **When** executo `vars`, **Then** todas as variáveis locais são listadas.
4. **Given** parado em `search`, **When** executo `regs`, **Then** os registradores x86-64 (RAX, RBX, RCX, RDX, RBP, RSP, RIP) são exibidos.
5. **Given** um ponteiro para nó, **When** executo `x <addr> 32`, **Then** um hex dump de 32 bytes a partir do endereço é exibido.

---

### User Story 5 — Pretty Printing of Rust Types (Priority: P2)

Como engenheiro de QA, quero que o rde-cli pretty-print corretamente os tipos padrão do Rust usados na implementação da árvore binária (`Option<Box<Node>>`, `Vec<i32>`, `String`, `Result`) e tipos customizados da árvore.

**Why this priority**: Pretty printing acelera a compreensão do estado da árvore, especialmente para estruturas aninhadas como `Option<Box<Node>>`.

**Independent Test**: Cada tipo pode ser inspecionado em um breakpoint isolado e o output comparado com o formato esperado.

**Acceptance Scenarios**:

1. **Given** uma variável `Option<Box<Node>>` com valor `Some(...)`, **When** executo `print root`, **Then** a saída mostra `Some(Node { ... })` com os campos do nó.
2. **Given** uma variável `Option<Box<Node>>` vazia (`None`), **When** executo `print left_child`, **Then** a saída é `None`.
3. **Given** um `Vec<i32>` usado em `inorder_traversal`, **When** executo `print result`, **Then** a saída é `[10, 20, 30, ...]`.
4. **Given** uma `String` usada como rótulo, **When** executo `print label`, **Then** a saída é `"tree-node-1"`.
5. **Given** um tipo customizado `TreeNode { value: i32, left: Option<Box<TreeNode>>, right: Option<Box<TreeNode>> }`, **When** executo `print node`, **Then** a estrutura completa é exibida até o limite de profundidade configurado.

---

### User Story 6 — Tree Navigation & Function Coverage (Priority: P1)

Como engenheiro de QA, quero depurar cada função da árvore binária individualmente, cobrindo todos os caminhos de execução (inserção em folha, inserção com rotação, busca que encontra, busca que falha, delete de folha, delete com um filho, delete com dois filhos).

**Why this priority**: A árvore binária é o alvo principal; todos os seus caminhos de execução devem ser exercitados sob debug.

**Independent Test**: Cada função pode ser testada isoladamente com um conjunto de dados específico que exercite todos os seus branches.

**Acceptance Scenarios**:

1. **Given** árvore vazia, **When** insiro o primeiro nó e depuro `insert`, **Then** o caminho de criação da raiz é coberto.
2. **Given** árvore com nós existentes, **When** insiro à esquerda e depuro, **Then** o caminho de inserção recursiva à esquerda é coberto.
3. **Given** árvore com nós existentes, **When** insiro à direita e depuro, **Then** o caminho de inserção recursiva à direita é coberto.
4. **Given** árvore desbalanceada (se AVL/RB), **When** insiro um nó que dispara rotação e depuro `rotate_left`/`rotate_right`, **Then** as rotações são executadas e inspecionáveis.
5. **Given** árvore com nós, **When** deleto uma folha, um nó com um filho, e um nó com dois filhos, **Then** todos os três caminhos de `delete` são depurados.

---

### User Story 7 — REPL Runtime Interaction (Priority: P2)

Como engenheiro de QA, quero executar comandos REPL em tempo de execução enquanto o processo está pausado, incluindo mudança de configurações, reavaliação de expressões e manipulação dinâmica de breakpoints.

**Why this priority**: O REPL é a interface primária do rde-cli; sua responsividade e consistência em tempo de execução são críticas.

**Independent Test**: Cada comando REPL pode ser testado isoladamente em um breakpoint verificando a saída imediata e o estado persistente.

**Acceptance Scenarios**:

1. **Given** processo pausado, **When** executo `set pretty-print on`, **Then** pretty printing é ativado e afeta comandos subsequentes.
2. **Given** processo pausado, **When** executo `set print-limit 5`, **Then** apenas 5 elementos são exibidos em coleções subsequentes.
3. **Given** processo pausado, **When** executo `set print-depth 2`, **Then** a profundidade de pretty print é limitada a 2 níveis.
4. **Given** processo pausado, **When** executo `set auto-disassemble on`, **Then** disassembly é exibido automaticamente em cada parada.
5. **Given** processo pausado, **When** executo `break delete` e depois `continue`, **Then** o novo breakpoint é atingido dinamicamente.

---

### User Story 8 — Thread, Module & Async Inspection (Priority: P2)

Como engenheiro de QA, quero inspecionar threads, módulos carregados e tasks Tokio (se aplicável) durante a execução da árvore binária para validar o tracking do sistema operacional.

**Why this priority**: Embora a árvore binária seja single-threaded, o debugger deve reportar corretamente o estado do processo e módulos carregados.

**Independent Test**: Cada comando de inspeção de sistema pode ser executado em qualquer breakpoint e comparado contra `tasklist`, `Process Hacker` ou output esperado.

**Acceptance Scenarios**:

1. **Given** sessão ativa, **When** executo `threads`, **Then** a thread principal é listada com estado `Suspended` ou `Running`.
2. **Given** sessão ativa, **When** executo `modules`, **Then** módulos como `ntdll.dll`, `kernel32.dll` e o executável principal são listados.
3. **Given** sessão ativa com Tokio (se aplicável), **When** executo `tasks`, **Then** as tasks async são listadas ou uma mensagem "No Tokio runtime detected" é exibida.
4. **Given** múltiplas threads (se debuggee threaded), **When** executo `thread <id>`, **Then** a thread selecionada muda e comandos subsequentes afetam essa thread.
5. **Given** sessão ativa, **When** executo `bt`, **Then** o backtrace da thread atual é exibido com símbolos resolvidos.

---

### User Story 9 — Memory & Disassembly Inspection (Priority: P2)

Como engenheiro de QA, quero inspecionar a memória bruta e o disassembly das funções da árvore binária para validar o nível baixo do debugger.

**Why this priority**: Inspeção de memória e disassembly são diferenciadores de um debugger nativo; devem funcionar corretamente para qualquer alvo.

**Independent Test**: Cada comando de inspeção de baixo nível pode ser testado em um breakpoint isolado comparando endereços e opcodes contra objdump ou similar.

**Acceptance Scenarios**:

1. **Given** parado em `insert`, **When** executo `disassemble` (ou auto-disassembly), **Then** as instruções assembly da função são exibidas.
2. **Given** parado em `insert`, **When** executo `x $rip 16`, **Then** 16 bytes a partir do RIP são exibidos em hex + ASCII.
3. **Given** um endereço de heap (ponteiro para nó), **When** executo `x <heap_addr> 64`, **Then** a estrutura do nó é visível em memória bruta.
4. **Given** parado após um `ModuleLoaded`, **When** executo `modules` seguido de disassembly no endereço base, **Then** o código do módulo é exibido.
5. **Given** breakpoint em uma instrução específica, **When** o hit ocorre e auto-disassembly está ligado, **Then** as instruções ao redor do RIP são automaticamente exibidas.

---

### User Story 10 — End-to-End Demo Scenarios (Priority: P1)

Como engenheiro de QA, quero executar cenários completos de demo que exercitem múltiplas features do rde-cli em sequência, simulando um fluxo de debug realista.

**Why this priority**: Cenários end-to-end validam a integração entre todos os componentes do debugger e a aplicação alvo.

**Independent Test**: Cada cenário de demo é um script de teste completo que pode ser executado de forma isolada e verificado contra golden paths.

**Acceptance Scenarios**:

1. **Given** a demo `--demo insert-sequence`, **When** depuro do início ao fim com breakpoints em cada insert, **Then** todos os nós são inspecionados e a árvore final é validada.
2. **Given** a demo `--demo search-miss`, **When** depuro a busca por um valor inexistente, **Then** o caminho de falha é percorrido com step into/out e variáveis inspecionadas.
3. **Given** a demo `--demo delete-rebalance`, **When** depuro a deleção que dispara rebalanceamento, **Then** as rotações são observadas via step e registros inspecionados.
4. **Given** a demo `--demo full-traversal`, **When** depuro inorder/preorder/postorder, **Then** a ordem de visitação é validada via print do vetor resultado.
5. **Given** a demo `--demo stress-test`, **When** insiro 100 nós aleatórios sob debug com breakpoints condicionais (se suportado), **Then** o debugger mantém performance e consistência.

## Edge Cases

- **What happens when** o usuário tenta definir um breakpoint em um símbolo inexistente (ex: `break nonexistent`)?
- **What happens when** o usuário tenta inspecionar memória em um endereço inválido?
- **What happens when** o debuggee termina inesperadamente (panic) e o debugger está em single step?
- **How does the system handle** múltiplos comandos REPL rápidos enquanto o processo está running?
- **How does the system handle** tentativa de `step` quando o processo já está no último frame de `main`?
- **What happens when** o usuário remove um breakpoint que está atualmente sendo "hit"?
- **How does the system handle** pretty-print de um `Option<Box<Node>>` com profundidade recursiva muito grande?
- **What happens when** o usuário tenta `continue` sem um processo ativo?

## Requirements *(mandatory)*

### Functional Requirements

#### FR-001: Setup do Alvo de Teste
O sistema DEVE fornecer uma aplicação `rust_app_example` escrita em Rust que implemente uma árvore binária de busca (BST) ou árvore balanceada (AVL/Red-Black), com as seguintes funções públicas: `new`, `insert`, `search`, `delete`, `find_min`, `find_max`, `height`, `size`, `inorder_traversal`, `preorder_traversal`, `postorder_traversal`, `is_empty`, `clear`. Se balanceada, também `rotate_left`, `rotate_right`, `balance_factor`. A aplicação DEVE aceitar o argumento `--demo <scenario>` para executar cenários predefinidos.

#### FR-002: Suite de Teste — Launch & Session (TC-001 a TC-015)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-001 | Launch standalone do binário | `rde-cli rust_app_example.exe` | `ProcessLaunched` com PID válido |
| TC-002 | Launch standalone com argumentos | `rde-cli rust_app_example.exe --demo insert` | Argumentos repassados, demo executa |
| TC-003 | Launch via Cargo (default) | `rde-cli cargo debug` | Target resolvido, build se stale, launch |
| TC-004 | Launch via Cargo com profile release | `rde-cli cargo debug --release` | Binário release carregado |
| TC-005 | Launch via Cargo com package/bin específicos | `rde-cli cargo debug --package tree --bin rust_app_example` | Target correto resolvido |
| TC-006 | Launch via Cargo com features | `rde-cli cargo debug --features "avl,tracing"` | Features compiladas e launch |
| TC-007 | Launch com TUI | `rde-cli --tui rust_app_example.exe` | Interface TUI inicia corretamente |
| TC-008 | Attach a processo running | `attach <pid>` no REPL | `ProcessAttached` com PID correto |
| TC-009 | Sessão termina com `quit` | `q` no REPL | Processo desanexado/terminado, CLI sai |
| TC-010 | Sessão termina quando debuggee exits | `continue` até o fim | `ProcessExited` com código 0 |
| TC-011 | Detect stale artifact e rebuild | Modificar src, `rde-cli cargo debug` | Build automático disparado |
| TC-012 | Cargo build failure handling | Introduzir erro de sintaxe, launch | Erro exibido, sessão não inicia |
| TC-013 | Launch de binário inexistente | `rde-cli nonexistent.exe` | Erro claro, sem crash |
| TC-014 | Launch de arquivo não-executável | `rde-cli README.md` | Erro claro, sem crash |
| TC-015 | Múltiplos launches na mesma sessão | Segundo launch após primeiro | Comportamento definido (rejeitar ou restart) |

#### FR-003: Suite de Teste — Breakpoints (TC-016 a TC-060)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-016 | Breakpoint por símbolo `main` | `break main` | ID retornado, hit em `main` |
| TC-017 | Breakpoint por símbolo `insert` | `break insert` | Hit ao entrar em `insert` |
| TC-018 | Breakpoint por símbolo `search` | `break search` | Hit ao entrar em `search` |
| TC-019 | Breakpoint por símbolo `delete` | `break delete` | Hit ao entrar em `delete` |
| TC-020 | Breakpoint por símbolo `find_min` | `break find_min` | Hit ao entrar em `find_min` |
| TC-021 | Breakpoint por símbolo `find_max` | `break find_max` | Hit ao entrar em `find_max` |
| TC-022 | Breakpoint por símbolo `height` | `break height` | Hit ao entrar em `height` |
| TC-023 | Breakpoint por símbolo `size` | `break size` | Hit ao entrar em `size` |
| TC-024 | Breakpoint por símbolo `inorder_traversal` | `break inorder_traversal` | Hit ao entrar |
| TC-025 | Breakpoint por símbolo `preorder_traversal` | `break preorder_traversal` | Hit ao entrar |
| TC-026 | Breakpoint por símbolo `postorder_traversal` | `break postorder_traversal` | Hit ao entrar |
| TC-027 | Breakpoint por símbolo `rotate_left` (se AVL) | `break rotate_left` | Hit ao entrar |
| TC-028 | Breakpoint por símbolo `rotate_right` (se AVL) | `break rotate_right` | Hit ao entrar |
| TC-029 | Breakpoint por símbolo `balance_factor` (se AVL) | `break balance_factor` | Hit ao entrar |
| TC-030 | Breakpoint por símbolo inexistente | `break nonexistent` | Mensagem de erro clara |
| TC-031 | Breakpoint por endereço hexadecimal | `break 0x140001000` | ID retornado, hit no endereço |
| TC-032 | Múltiplos breakpoints simultâneos | `break main`, `break insert` | Ambos ativos, hits individuais reportados |
| TC-033 | Deleção de breakpoint por ID | `delbreak 1` | Breakpoint removido, não dispara mais |
| TC-034 | Deleção de breakpoint inexistente | `delbreak 999` | Mensagem de erro clara |
| TC-035 | Deleção de todos os breakpoints | `delbreak` em todos | Lista vazia, continue executa até o fim |
| TC-036 | Breakpoint em tempo de execução (dinâmico) | Adicionar `break insert` enquanto running | Breakpoint ativado e hit subsequente |
| TC-037 | Remoção de breakpoint em tempo de execução | Remover `break insert` enquanto running | Breakpoint desativado imediatamente |
| TC-038 | Breakpoint em recursão profunda (insert) | `break insert` com árvore profunda | Hit em cada frame recursivo |
| TC-039 | Breakpoint em função chamada múltiplas vezes | `break search` em loop | Hit múltiplas vezes, IDs consistentes |
| TC-040 | Breakpoint hit mostra thread ID | Hit em `main` | Mensagem inclui `Thread <TID>` |
| TC-041 | Breakpoint hit mostra endereço | Hit em `insert` | Mensagem inclui endereço hex |
| TC-042 | Breakpoint na demo `--demo insert-sequence` | `break insert` + demo | Hits em cada inserção |
| TC-043 | Breakpoint na demo `--demo delete-rebalance` | `break delete` + demo | Hit antes de cada deleção |
| TC-044 | Breakpoint na demo `--demo search-miss` | `break search` + demo | Hit na busca |
| TC-045 | Breakpoint na demo `--demo full-traversal` | `break inorder_traversal` + demo | Hit no início do traversal |
| TC-046 | Breakpoint em `println!` macro | `break std::io::_print` (se resolvido) | Hit ou erro claro se símbolo não encontrado |
| TC-047 | Breakpoint em `Drop` de `Box<Node>` | `break <Drop>` | Hit quando nó é desalocado |
| TC-048 | Breakpoint em função auxiliar privada | `break Tree::rebalance` (se existir) | Hit na função privada |
| TC-049 | Breakpoint duplicado no mesmo símbolo | `break insert` duas vezes | Dois IDs diferentes, ambos funcionam |
| TC-050 | Breakpoint com símbolo parcial | `break ins` | Comportamento definido (match ou erro) |
| TC-051 | Breakpoint em `main` com argumentos | `break main` + `--demo insert` | Hit em `main`, args inspecionáveis |
| TC-052 | Breakpoint em `new` | `break Tree::new` | Hit na construção |
| TC-053 | Breakpoint em `clear` | `break clear` | Hit na limpeza |
| TC-054 | Breakpoint em `is_empty` | `break is_empty` | Hit no check |
| TC-055 | Breakpoint em `std::cmp::PartialEq` (se usado) | `break eq` | Hit nas comparações |
| TC-056 | Breakpoint no system initial breakpoint | Launch padrão | `BreakpointHit` do tipo `SystemInitial` tratado corretamente |
| TC-057 | Breakpoint hit com auto-disassembly on | `set auto-disassemble on` + hit | Disassembly exibido automaticamente |
| TC-058 | Breakpoint hit com auto-disassembly off | `set auto-disassemble off` + hit | Sem disassembly automático |
| TC-059 | Breakpoint hit após `continue` longo | `continue` após hit anterior | Processo para no próximo hit |
| TC-060 | Breakpoint hit após `step` curto | `step` de perto | Processo para no próximo hit |

#### FR-004: Suite de Teste — Execution Control (TC-061 a TC-120)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-061 | Continue após breakpoint em `main` | `continue` | Executa até próximo hit ou fim |
| TC-062 | Continue após breakpoint em `insert` | `continue` | Executa até próximo hit ou fim |
| TC-063 | Continue após breakpoint em `delete` | `continue` | Executa até próximo hit ou fim |
| TC-064 | Continue sem breakpoint seguinte | `continue` do último breakpoint | `ProcessExited` |
| TC-065 | Step into em `main` chamando `demo` | `step` em `main` | Entra na função `demo` |
| TC-066 | Step into em `demo` chamando `insert` | `step` em `demo` | Entra em `insert` |
| TC-067 | Step into em `insert` chamando `insert` recursivo | `step` em `insert` | Desce um nível de recursão |
| TC-068 | Step into em `insert` chamando `rotate_left` | `step` | Entra em `rotate_left` |
| TC-069 | Step into em `insert` chamando `rotate_right` | `step` | Entra em `rotate_right` |
| TC-070 | Step into em `delete` chamando `find_min` | `step` | Entra em `find_min` |
| TC-071 | Step into em `delete` chamando `delete` recursivo | `step` | Desce um nível de recursão |
| TC-072 | Step into em `search` chamando `search` recursivo | `step` | Desce um nível de recursão |
| TC-073 | Step into em `height` chamando `height` recursivo | `step` | Desce um nível de recursão |
| TC-074 | Step into em `inorder_traversal` recursivo | `step` | Desce um nível |
| TC-075 | Step into em `preorder_traversal` recursivo | `step` | Desce um nível |
| TC-076 | Step into em `postorder_traversal` recursivo | `step` | Desce um nível |
| TC-077 | Step into em `Box::new` dentro de `insert` | `step` | Entra no alloc (ou comportamento definido) |
| TC-078 | Step into em `drop` de `Box<Node>` | `step` | Entra no dealloc (ou comportamento definido) |
| TC-079 | Step over em chamada recursiva de `insert` | step over | Para após retorno da recursão |
| TC-080 | Step over em chamada recursiva de `delete` | step over | Para após retorno da recursão |
| TC-081 | Step over em chamada recursiva de `search` | step over | Para após retorno da recursão |
| TC-082 | Step over em chamada recursiva de `height` | step over | Para após retorno da recursão |
| TC-083 | Step over em chamada a `rotate_left` | step over | Para após retorno da rotação |
| TC-084 | Step over em chamada a `rotate_right` | step over | Para após retorno da rotação |
| TC-085 | Step over em chamada a `find_min` | step over | Para após retorno |
| TC-086 | Step over em chamada a `find_max` | step over | Para após retorno |
| TC-087 | Step over em `println!` | step over | Para após output |
| TC-088 | Step over em `Vec::push` | step over | Para após push |
| TC-089 | Step over em `Option::take` | step over | Para após take |
| TC-090 | Step out de `rotate_left` para `insert` | step out | Retorna ao chamador `insert` |
| TC-091 | Step out de `rotate_right` para `insert` | step out | Retorna ao chamador `insert` |
| TC-092 | Step out de `find_min` para `delete` | step out | Retorna ao chamador `delete` |
| TC-093 | Step out de recursão `insert` para nível superior | step out | Sobe um nível |
| TC-094 | Step out de recursão `delete` para nível superior | step out | Sobe um nível |
| TC-095 | Step out de recursão `search` para nível superior | step out | Sobe um nível |
| TC-096 | Step out de `height` recursivo para nível superior | step out | Sobe um nível |
| TC-097 | Step out de `main` | step out | Comportamento definido (fim ou erro claro) |
| TC-098 | Step into em `std` function (ex: `cmp::max`) | `step` | Comportamento definido (entra ou step over implícito) |
| TC-099 | Step into em função inline | `step` | Comportamento definido |
| TC-100 | Continue após step sequence | `continue` após vários steps | Executa normalmente |
| TC-101 | Step sequence: main → demo → insert → rotate | 3× `step` | RIP avança conforme esperado |
| TC-102 | Step sequence: insert recursivo 5 níveis | 5× `step` | 5 frames de recursão observados |
| TC-103 | Step over sequence em loop de inorder | 3× step over | Cada iteração coberta sem entrar na recursão |
| TC-104 | Step out sequence de profundidade 5 | 5× step out | Retorna 5 níveis até o topo |
| TC-105 | Mixed control: step, continue, step | `step`, `continue`, `step` | Transições corretas |
| TC-106 | Mixed control: continue, step into | `continue`, `step` | Para no hit, depois step funciona |
| TC-107 | Mixed control: breakpoint hit, step out | hit, `step out` | Sai da função atual |
| TC-108 | Mixed control: breakpoint hit, continue | hit, `continue` | Prossegue até próximo |
| TC-109 | Execution control em `main` vazio | `step` em `main` que só chama demo | Avança para `demo` |
| TC-110 | Execution control em função sem chamadas | `step` em `find_min` folha | Avança para próxima instrução ou retorna |
| TC-111 | Continue após system initial breakpoint | `continue` após launch | Processo executa além de ntdll |
| TC-112 | Step após system initial breakpoint | `step` após launch | Single step funciona a partir de ntdll |
| TC-113 | Continue com múltiplos breakpoints | `continue` com 5 breakpoints | Para em cada um na ordem correta |
| TC-114 | Continue após deleção de breakpoint ativo | `delbreak` no hit atual, `continue` | Prossegue sem re-hit imediato |
| TC-115 | Continue após modificação de breakpoint | Re-add breakpoint, `continue` | Hit no novo endereço |
| TC-116 | Step into com breakpoint no destino | `step` para função com breakpoint | Para tanto no step quanto no breakpoint |
| TC-117 | Step over com breakpoint dentro da função chamada | step over para função com breakpoint | Comportamento definido (para ou não) |
| TC-118 | Step out com breakpoint no retorno | step out para endereço com breakpoint | Para no breakpoint de retorno |
| TC-119 | Execution control durante demo `--demo stress-test` | `continue` com 100 inserts | Performance aceitável (< 5s por continue) |
| TC-120 | Execution control com processo que panics | `continue` até panic | Evento `Exception` ou `ProcessExited` com código não-zero |

#### FR-005: Suite de Teste — Variable Inspection (TC-121 a TC-160)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-121 | Print de parâmetro `value: i32` em `insert` | `print value` | Valor inteiro correto exibido |
| TC-122 | Print de parâmetro `value: i32` em `search` | `print value` | Valor inteiro correto exibido |
| TC-123 | Print de parâmetro `value: i32` em `delete` | `print value` | Valor inteiro correto exibido |
| TC-124 | Print de `root: Option<Box<Node>>` em `insert` | `print root` | Estrutura do nó exibida |
| TC-125 | Print de `root: Option<Box<Node>>` em `search` | `print root` | Estrutura do nó exibida |
| TC-126 | Print de `root: Option<Box<Node>>` em `delete` | `print root` | Estrutura do nó exibida |
| TC-127 | Print de `node: &Node` em `height` | `print node` | Campos do nó exibidos |
| TC-128 | Print de `node: &Node` em `size` | `print node` | Campos do nó exibidos |
| TC-129 | Print de `left: Option<Box<Node>>` | `print left` | `Some(...)` ou `None` |
| TC-130 | Print de `right: Option<Box<Node>>` | `print right` | `Some(...)` ou `None` |
| TC-131 | Print de `result: Vec<i32>` em `inorder_traversal` | `print result` | Vetor ordenado exibido |
| TC-132 | Print de `result: Vec<i32>` em `preorder_traversal` | `print result` | Vetor na ordem pré-fixada |
| TC-133 | Print de `result: Vec<i32>` em `postorder_traversal` | `print result` | Vetor na ordem pós-fixada |
| TC-134 | Print de `min_value: i32` em `find_min` | `print min_value` | Valor mínimo correto |
| TC-135 | Print de `max_value: i32` em `find_max` | `print max_value` | Valor máximo correto |
| TC-136 | Print de `current_height: i32` em `height` | `print current_height` | Altura correta |
| TC-137 | Print de `tree_size: i32` em `size` | `print tree_size` | Tamanho correto |
| TC-138 | Print de variável não inicializada (se visível) | `print uninitialized` | Comportamento definido (erro ou valor lixo) |
| TC-139 | Print de expressão `value + 1` | `print value + 1` | Resultado da expressão exibido |
| TC-140 | Print de ponteiro bruto `*const Node` | `print raw_ptr` | Endereço hex exibido |
| TC-141 | Print com flag `--raw` | `print --raw root` | Memória bruta exibida |
| TC-142 | Print com flag `--limit 2` | `print --limit 2 result` | Apenas 2 elementos do vetor |
| TC-143 | Print com flag `--depth 1` | `print --depth 1 root` | Apenas 1 nível de profundidade |
| TC-144 | `vars` em `main` | `vars` | `argc`, `argv`, variáveis locais listadas |
| TC-145 | `vars` em `insert` | `vars` | `value`, `root`, variáveis locais listadas |
| TC-146 | `vars` em `delete` | `vars` | `value`, `root`, variáveis locais listadas |
| TC-147 | `vars` em `search` | `vars` | `value`, `root`, variáveis locais listadas |
| TC-148 | `vars` em `rotate_left` | `vars` | Variáveis de rotação listadas |
| TC-149 | `vars` em `rotate_right` | `vars` | Variáveis de rotação listadas |
| TC-150 | `vars` com pretty-print off | `set pretty-print off`, `vars` | Output raw |
| TC-151 | `regs` em `main` | `regs` | RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP, RIP, RFLAGS exibidos |
| TC-152 | `regs` em `insert` | `regs` | Registradores exibidos com valores coerentes |
| TC-153 | `regs` após `step` | `regs` | RIP avançou conforme esperado |
| TC-154 | `regs` após `continue` | `regs` em breakpoint | RIP no endereço do breakpoint |
| TC-155 | `bt` (backtrace) em `main` | `bt` | 1 frame: `main` |
| TC-156 | `bt` em `insert` recursivo profundo | `bt` | N frames de recursão visíveis |
| TC-157 | `bt` em `rotate_left` chamado por `insert` | `bt` | `rotate_left` → `insert` → `demo` → `main` |
| TC-158 | `bt` após `step into` | `bt` | Frame adicionado |
| TC-159 | `bt` após `step out` | `bt` | Frame removido |
| TC-160 | `bt` com símbolos resolvidos | `bt` | Nomes de função visíveis, não apenas endereços |

#### FR-006: Suite de Teste — Pretty Printing (TC-161 a TC-185)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-161 | Pretty print `Option<Box<Node>>` = `Some` | `print root` | `Some(Node { value: 10, left: None, right: None })` |
| TC-162 | Pretty print `Option<Box<Node>>` = `None` | `print left` | `None` |
| TC-163 | Pretty print `Vec<i32>` vazio | `print result` | `[]` |
| TC-164 | Pretty print `Vec<i32>` com 3 elementos | `print result` | `[10, 20, 30]` |
| TC-165 | Pretty print `Vec<i32>` com 150 elementos (limite padrão) | `print result` | `[...]` truncado ou 100 primeiros |
| TC-166 | Pretty print `String` | `print label` | `"hello"` |
| TC-167 | Pretty print `String` vazia | `print empty_label` | `""` |
| TC-168 | Pretty print `Result<i32, String>` = `Ok` | `print res` | `Ok(42)` |
| TC-169 | Pretty print `Result<i32, String>` = `Err` | `print res` | `Err("fail")` |
| TC-170 | Pretty print `HashMap<i32, String>` (summary) | `print map` | `HashMap { ... }` ou similar |
| TC-171 | Pretty print `Node` com filhos | `print node` | `Node { value: 20, left: Some(...), right: Some(...) }` |
| TC-172 | Pretty print `Node` com profundidade 3 | `print --depth 3 root` | 3 níveis de árvore exibidos |
| TC-173 | Pretty print `Node` com profundidade 1 | `print --depth 1 root` | `Node { value: 20, left: Some(...), right: Some(...) }` sem expandir |
| TC-174 | Pretty print com `set print-limit 5` | `set print-limit 5`, `print vec` | 5 elementos |
| TC-175 | Pretty print com `set print-depth 2` | `set print-depth 2`, `print root` | 2 níveis |
| TC-176 | Pretty print com `set pretty-print off` | `set pretty-print off`, `print root` | Output raw/hex |
| TC-177 | Pretty print de struct customizada `TreeStats` | `print stats` | Campos exibidos |
| TC-178 | Pretty print de enum customizado `TreeError` | `print err` | Variante exibida |
| TC-179 | Pretty print de tupla `(i32, String)` | `print tuple` | `(10, "hello")` |
| TC-180 | Pretty print de array `[i32; 5]` | `print arr` | `[1, 2, 3, 4, 5]` |
| TC-181 | Pretty print de referência `&Node` | `print node_ref` | Valor referenciado exibido |
| TC-182 | Pretty print de `Box<Node>` | `print boxed` | Conteúdo do box exibido |
| TC-183 | Pretty print com budget excedido | `print --depth 10 root` muito profunda | `Truncated` ou mensagem de limite |
| TC-184 | Pretty print de `bool` | `print flag` | `true` ou `false` |
| TC-185 | Pretty print de `i64` | `print big_num` | Valor correto sem truncamento |

#### FR-007: Suite de Teste — REPL Runtime Commands (TC-186 a TC-210)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-186 | `set pretty-print on` em runtime | `set pretty-print on` | Configuração ativada |
| TC-187 | `set pretty-print off` em runtime | `set pretty-print off` | Configuração desativada |
| TC-188 | `set print-limit 10` em runtime | `set print-limit 10` | Limite alterado para 10 |
| TC-189 | `set print-limit 1000` em runtime | `set print-limit 1000` | Limite alterado para 1000 |
| TC-190 | `set print-depth 1` em runtime | `set print-depth 1` | Profundidade alterada |
| TC-191 | `set print-depth 10` em runtime | `set print-depth 10` | Profundidade alterada |
| TC-192 | `set auto-disassemble on` em runtime | `set auto-disassemble on` | Auto-disassembly ativado |
| TC-193 | `set auto-disassemble off` em runtime | `set auto-disassemble off` | Auto-disassembly desativado |
| TC-194 | `set disassembly-count 5` em runtime | `set disassembly-count 5` | Contagem alterada |
| TC-195 | `set disassembly-count 50` em runtime | `set disassembly-count 50` | Contagem alterada |
| TC-196 | Comando inválido no REPL | `foobar` | Mensagem de erro/ajuda |
| TC-197 | Comando vazio no REPL | `<enter>` | Nenhum erro, prompt retorna |
| TC-198 | Breakpoint dinâmico em runtime | `break insert` durante pausa | Breakpoint adicionado |
| TC-199 | Deleção dinâmica em runtime | `delbreak 1` durante pausa | Breakpoint removido |
| TC-200 | Print de expressão em runtime | `print value` durante pausa | Valor exibido |
| TC-201 | Vars em runtime | `vars` durante pausa | Variáveis listadas |
| TC-202 | Regs em runtime | `regs` durante pausa | Registradores exibidos |
| TC-203 | Memory examine em runtime | `x $rsp 16` durante pausa | Hex dump exibido |
| TC-204 | Backtrace em runtime | `bt` durante pausa | Stack trace exibido |
| TC-205 | Threads em runtime | `threads` durante pausa | Threads listadas |
| TC-206 | Modules em runtime | `modules` durante pausa | Módulos listados |
| TC-207 | Tasks em runtime | `tasks` durante pausa | Tasks listadas ou mensagem apropriada |
| TC-208 | Disassemble em runtime | `disassemble` durante pausa | Assembly exibido |
| TC-209 | Multiple REPL commands sequence | `print value`, `regs`, `bt` | Todas as saídas corretas |
| TC-210 | REPL command após `continue` com hit rápido | `continue` + hit imediato + comando | Comando executa sem deadlock |

#### FR-008: Suite de Teste — Thread, Module & Task Inspection (TC-211 a TC-230)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-211 | `threads` com 1 thread | `threads` | Thread principal listada |
| TC-212 | `threads` com debuggee threaded | `threads` | Múltiplas threads listadas |
| TC-213 | `thread <id>` para thread principal | `thread <main_tid>` | Thread selecionada |
| TC-214 | `thread <id>` para thread worker | `thread <worker_tid>` | Thread selecionada |
| TC-215 | `thread` com ID inválido | `thread 99999` | Erro claro |
| TC-216 | `modules` após launch | `modules` | `ntdll.dll`, `kernel32.dll`, executável listados |
| TC-217 | `modules` após `ModuleLoaded` | `modules` | Novo módulo aparece na lista |
| TC-218 | `tasks` sem Tokio | `tasks` | "No Tokio runtime detected" |
| TC-219 | `tasks` com Tokio (se aplicável) | `tasks` | Tabela com ID, State, Function, Thread |
| TC-220 | `bt` em thread principal | `bt` | Stack trace da main thread |
| TC-221 | `bt` em thread worker | Selecionar worker, `bt` | Stack trace do worker |
| TC-222 | `regs` em thread principal | `regs` | Registradores da main thread |
| TC-223 | `regs` em thread worker | Selecionar worker, `regs` | Registradores do worker |
| TC-224 | `print` em thread worker | Selecionar worker, `print var` | Variável do worker exibida |
| TC-225 | `modules` lista endereços base | `modules` | Endereços hex visíveis |
| TC-226 | `modules` lista nomes | `modules` | Nomes de DLL e executável visíveis |
| TC-227 | `threads` lista estados | `threads` | Estados (Running, Suspended) visíveis |
| TC-228 | `threads` marca thread selecionada | `threads` | `*` na thread atual |
| TC-229 | ThreadCreated event | Launch de threaded debuggee | Evento `ThreadCreated` recebido |
| TC-230 | ThreadExited event | Continue threaded debuggee | Evento `ThreadExited` recebido |

#### FR-009: Suite de Teste — Memory & Disassembly (TC-231 a TC-250)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-231 | `x <addr> 16` em endereço de código | `x $rip 16` | 16 bytes de código em hex/ASCII |
| TC-232 | `x <addr> 64` em endereço de heap | `x <heap_ptr> 64` | 64 bytes do nó em hex/ASCII |
| TC-233 | `x <addr> 32` em endereço de stack | `x $rsp 32` | 32 bytes da stack |
| TC-234 | `x` com endereço inválido | `x 0x1 16` | Erro claro (access denied/invalid) |
| TC-235 | `x` com size 0 | `x $rip 0` | Comportamento definido |
| TC-236 | `x` com size muito grande | `x $rip 1000000` | Comportamento definido (truncado ou erro) |
| TC-237 | Disassembly em `main` | `disassemble` | Instruções de `main` |
| TC-238 | Disassembly em `insert` | `disassemble` | Instruções de `insert` |
| TC-239 | Disassembly em `delete` | `disassemble` | Instruções de `delete` |
| TC-240 | Disassembly em `search` | `disassemble` | Instruções de `search` |
| TC-241 | Disassembly com count 5 | `set disassembly-count 5`, `disassemble` | 5 instruções |
| TC-242 | Disassembly com count 50 | `set disassembly-count 50`, `disassemble` | 50 instruções |
| TC-243 | Auto-disassembly on hit | `set auto-disassemble on`, hit | Assembly exibido automaticamente |
| TC-244 | Auto-disassembly off hit | `set auto-disassemble off`, hit | Sem assembly automático |
| TC-245 | Disassembly por símbolo | `disassemble insert` (se suportado) | Instruções de `insert` |
| TC-246 | Disassembly por endereço | `disassemble 0x140001000` (se suportado) | Instruções no endereço |
| TC-247 | Memory bytes pretty-printed | `x <vec_ptr> 24` com pretty-print | Vec interpretado se possível |
| TC-248 | Memory examine de `Node` bruto | `x <node_ptr> 24` | Layout memória do nó visível |
| TC-249 | Memory examine após `Box::new` | `x <new_ptr> 24` | Novo nó alocado visível |
| TC-250 | Memory examine após `drop` | `x <old_ptr> 24` | Comportamento definido (memória liberada ou lixo) |

#### FR-010: Suite de Teste — Integration & End-to-End (TC-251 a TC-270)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-251 | E2E: Launch → break main → continue → exit | `break main`, `continue` | Sequência completa sem erros |
| TC-252 | E2E: Launch → break insert → print → continue → exit | `break insert`, `print value`, `continue` | Estado inspecionado corretamente |
| TC-253 | E2E: Demo `--demo insert-sequence` completo | Launch + demo | Todos os inserts depuráveis |
| TC-254 | E2E: Demo `--demo search-miss` completo | Launch + demo | Caminho de falha depurável |
| TC-255 | E2E: Demo `--demo delete-rebalance` completo | Launch + demo | Rotações observáveis |
| TC-256 | E2E: Demo `--demo full-traversal` completo | Launch + demo | Traversais depuráveis |
| TC-257 | E2E: Demo `--demo stress-test` completo | Launch + demo | 100 inserts sem crash do debugger |
| TC-258 | E2E: Launch → 5 breakpoints → continue sequence | 5 breaks + continues | Para em cada um |
| TC-259 | E2E: Launch → step into all functions | Step into em cada função | Cada função entrada observada |
| TC-260 | E2E: Launch → step over all functions | Step over em cada função | Cada função coberta sem entrar |
| TC-261 | E2E: REPL script completo (sem interação) | Sequência de comandos pré-definida | Output consistente com golden path |
| TC-262 | E2E: Golden path match | Comparar event stream | Match com `demo_success.txt` ou novo golden path |
| TC-263 | E2E: Cargo debug → stale check → build → debug | Modificar src, `cargo debug` | Rebuild e debug automáticos |
| TC-264 | E2E: Breakpoint → hit → REPL command → continue → exit | Comandos durante pausa | Fluxo completo |
| TC-265 | E2E: Multiple REPL commands entre hits | Vários prints/regs entre continues | Todos os comandos respondem |
| TC-266 | E2E: Debugger sobrevive a panic do debuggee | `continue` até panic | Debugger permanece ativo ou sai graciosamente |
| TC-267 | E2E: Debuggee com threads → inspeção completa | Launch threaded + threads/modules/tasks | Todas as inspeções funcionam |
| TC-268 | E2E: Session restart (quit + relaunch) | `q`, depois novo launch | Nova sessão limpa |
| TC-269 | E2E: TUI mode launch → quit | `rde-cli --tui` + quit | TUI inicia e termina sem crash |
| TC-270 | E2E: Full regression suite | Todos os TCs acima executados | 100% pass rate |

#### FR-011: Suite de Teste — Error Handling & Edge Cases (TC-271 a TC-290)

| ID | Cenário de Teste | Comando(s) | Critério de Sucesso |
|----|------------------|------------|---------------------|
| TC-271 | Breakpoint em símbolo inválido | `break xyz123` | Erro: símbolo não encontrado |
| TC-272 | Deleção de breakpoint inexistente | `delbreak 999` | Erro: breakpoint não encontrado |
| TC-273 | Print de variável inexistente | `print nonexistent` | Erro: variável não encontrada |
| TC-274 | Step quando processo running | `step` sem pausa | Erro: processo não pausado |
| TC-275 | Continue quando processo running | `continue` sem pausa | Erro: já running ou ignora |
| TC-276 | Comando sem sessão ativa | `break main` sem launch | Erro: nenhum processo ativo |
| TC-277 | Memory examine em endereço nulo | `x 0x0 16` | Erro de acesso |
| TC-278 | Attach a PID inexistente | `attach 99999` | Erro: processo não encontrado |
| TC-279 | Attach a PID sem permissão | `attach 4` (system) | Erro: acesso negado |
| TC-280 | Pretty print de tipo não suportado | `print custom_struct` | Output raw ou mensagem informativa |
| TC-281 | REPL command muito longo | Comando com 1000+ caracteres | Comportamento definido |
| TC-282 | Unicode em strings pretty-printed | `print unicode_string` | Caracteres corretos exibidos |
| TC-283 | Negative numbers pretty-printed | `print negative_val` | `-42` exibido corretamente |
| TC-284 | Float pretty-printed (se usado) | `print float_val` | `3.14` exibido corretamente |
| TC-285 | Empty Vec pretty-printed | `print empty_vec` | `[]` exibido |
| TC-286 | Vec de 1 elemento pretty-printed | `print single_vec` | `[42]` exibido |
| TC-287 | Nested Option (Option<Option<Node>>) | `print nested` | `Some(Some(...))` ou `Some(None)` |
| TC-288 | Debugger performance com 50 breakpoints | 50 breaks + continue | < 2s por operação |
| TC-289 | Debugger performance com 1000 nodes | Debug de árvore grande | < 5s para percorrer |
| TC-290 | Debugger memory stability (long session) | Sessão de 30 min | Sem vazamento de memória |

### Key Entities

- **BinaryTree (BST/AVL)**: Estrutura de dados principal com raiz `Option<Box<Node>>`. Atributos: `root`, métodos: `insert`, `search`, `delete`, `find_min`, `find_max`, `height`, `size`, `inorder_traversal`, `preorder_traversal`, `postorder_traversal`, `is_empty`, `clear`.
- **TreeNode**: Nó individual da árvore. Atributos: `value: i32`, `left: Option<Box<TreeNode>>`, `right: Option<Box<TreeNode>>`, `height: i32` (se AVL).
- **DebuggeeProcess**: Processo Windows sendo depurado. Atributos: `pid`, `threads`, `modules`, `breakpoints`.
- **Breakpoint**: Ponto de parada. Atributos: `id`, `address` ou `symbol`, `enabled`.
- **DebuggerSession**: Estado da sessão de debug. Atributos: `process`, `config` (pretty-print, limits), `selected_thread`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Todos os 290 casos de teste (TC-001 a TC-290) são executáveis via CLI e documentados com comando exato, estado inicial e critério de sucesso.
- **SC-002**: A aplicação `rust_app_example` compila sem warnings críticos e executa a demo em menos de 3 segundos sem debugger.
- **SC-003**: O debugger consegue percorrer todas as funções da árvore binária (mínimo 15 funções públicas) com step into/out/over sem crash.
- **SC-004**: Pretty printing funciona para `Option<Box<Node>>` com profundidade configurável e exibe corretamente `Some`/`None` e filhos.
- **SC-005**: A suite de testes end-to-end (TC-251 a TC-270) executa com 100% de pass rate em ambiente Windows 10/11 x64.
- **SC-006**: O debugger mantém performance: tempo entre `continue` e próximo breakpoint hit é proporcional ao tempo de execução nativa (overhead < 50%).
- **SC-007**: Casos de teste de erro (TC-271 a TC-290) produzem mensagens de erro claras e não causam crash do debugger.
- **SC-008**: A especificação gera um golden path (`demo_success.txt` ou equivalente) que pode ser usado para regressão automatizada.

## Assumptions

- A aplicação `rust_app_example` será criada como um projeto Cargo separado ou exemplo dentro do workspace, compilável com `cargo build`.
- A árvore binária é uma BST clássica ou AVL (se balanceamento for requerido). Red-Black Tree é considerada fora de escopo a menos que explicitamente solicitada.
- O ambiente de execução é Windows 10/11 x86-64, conforme documentado nas guidelines do RDE.
- O rde-cli já possui as funcionalidades listadas (continue, step, print, break, etc.) implementadas conforme as specs anteriores (001-005).
- Tokio tasks inspection é testada apenas se o debuggee usar Tokio; para a árvore binária pura, a expectativa é a mensagem "No Tokio runtime detected".
- Step out pode não ser suportado na versão atual do engine; se não for, os casos de teste relacionados devem ser marcados como `pending` ou `skipped`.
- O número total de casos de teste (290) cobre todas as combinações principais; casos adicionais podem ser adicionados durante a implementação.
