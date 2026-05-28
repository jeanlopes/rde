# Feature Specification: Rust Debug Extensions

**Feature Branch**: `005-rust-debug-extensions`

**Created**: 2026-05-28

**Status**: Draft

**Input**: User description: "005 rust-debug-extensions 9 Diferenciais Rust: pretty printers, async tasks, cargo integration Visualizar Option, Vec, tasks tokio, cargo debug."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Visualizar valores Rust complexos com pretty printers (Priority: P1)

Um desenvolvedor Rust está debugando uma aplicação Windows nativa com RDE. Quando a execução para em um breakpoint, o desenvolvedor inspeciona variáveis locais e quer ver representações legíveis de tipos padrão Rust (`Option<T>`, `Vec<T>`, `String`, `HashMap<K,V>`) em vez de layouts de memória brutos com ponteiros e offsets.

**Why this priority**: Pretty printers são o diferencial mais visível de um debugger Rust. Sem eles, o desenvolvedor precisa manualmente interpretar a representação interna de `Option` ou `Vec`, o que torna o debugging inutilizável para tipos Rust.

**Independent Test**: Pode ser testado isoladamente configurando um breakpoint em código que use `Option<i32>`, `Vec<String>` e `HashMap<u32, u32>`, e verificando que a saída do comando `print` ou `vars` exibe valores legíveis (`Some(42)`, lista de strings, pares chave-valor).

**Acceptance Scenarios**:

1. **Given** um processo debugado parado em um breakpoint com uma variável `opt: Option<i32> = Some(42)`, **When** o desenvolvedor inspeciona a variável, **Then** o sistema exibe `Some(42)` em vez do layout de memória da enum.
2. **Given** uma variável `v: Vec<i32>` com elementos `[10, 20, 30]`, **When** o desenvolvedor inspeciona a variável, **Then** o sistema exibe `[10, 20, 30]` com índices e valores visíveis.
3. **Given** uma variável `s: String` contendo `"hello"`, **When** o desenvolvedor inspeciona a variável, **Then** o sistema exibe `"hello"` em vez do ponteiro para `str` e capacidade/len.

---

### User Story 2 - Inspecionar tarefas assíncronas do Tokio (Priority: P2)

Um desenvolvedor está debugando uma aplicação async que usa Tokio. O desenvolvedor quer entender o estado de execução das tasks async — quais estão rodando, quais estão esperando por I/O ou timers, e de onde foram spawnadas — para diagnosticar deadlocks, starvation ou comportamento inesperado de concorrência.

**Why this priority**: Aplicações Rust modernas usam async/await extensivamente. Um debugger Rust que não entende o runtime Tokio deixa o desenvolvedor "cego" em relação à concorrência, forçando-o a inferir o estado async apenas pela stack trace da thread do runtime.

**Independent Test**: Pode ser testado isoladamente spawnando múltiplas tasks Tokio em um programa debugável, parando em um breakpoint, e verificando que o comando `tasks` ou equivalente lista as tasks com seus estados e nomes/IDs.

**Acceptance Scenarios**:

1. **Given** um processo debugado com Tokio runtime ativo e 3 tasks spawnadas, **When** o desenvolvedor executa o comando de inspeção de tasks, **Then** o sistema lista as 3 tasks com seus respectivos estados (e.g., running, idle, sleeping).
2. **Given** uma task async spawnada com `tokio::spawn(foo())`, **When** o desenvolvedor inspeciona a lista de tasks, **Then** o sistema exibe o nome da função `foo` (ou identificador derivado dos símbolos) associado à task.
3. **Given** uma task que completou sua execução, **When** o desenvolvedor inspeciona a lista de tasks, **Then** o sistema indica que a task está no estado completed ou a remove da lista ativa.

---

### User Story 3 - Integração com Cargo para debug facilitado (Priority: P3)

Um desenvolvedor quer iniciar uma sessão de debug a partir de um projeto Cargo existente, sem precisar manualmente encontrar o caminho do binário compilado em `target/debug/` ou `target/release/`. O sistema detecta que está em um diretório Cargo, resolve o target correto (binário padrão ou especificado), e inicia o debuggee automaticamente.

**Why this priority**: Reduz o atrito de iniciar uma sessão de debug. Desenvolvedores Rust esperam que o debugger entenda o ecossistema Cargo, assim como `cargo test` e `cargo run` entendem.

**Independent Test**: Pode ser testado isoladamente em um diretório com `Cargo.toml` válido, executando o comando de debug Cargo do RDE e verificando que o binário correto é carregado e executado.

**Acceptance Scenarios**:

1. **Given** um diretório com `Cargo.toml` contendo um binário padrão, **When** o desenvolvedor inicia o debug via integração Cargo, **Then** o sistema compila (se necessário) e carrega o binário correto de `target/debug/`.
2. **Given** um workspace Cargo com múltiplos pacotes, **When** o desenvolvedor especifica um pacote, **Then** o sistema resolve e carrega o binário do pacote correto.
3. **Given** um projeto Cargo com features ativas, **When** o desenvolvedor inicia o debug, **Then** o sistema aciona `cargo build` automaticamente se o binário estiver ausente ou mais antigo que o código-fonte, respeitando o profile e features configurados.

---

### Edge Cases

- **Vec muito grande**: Como o sistema lida com `Vec` contendo milhares ou milhões de elementos? Deve haver um limite de exibição paginado.
- **Option com tipo complexo aninhado**: `Option<Vec<Option<String>>>` deve ser pretty-printed recursivamente até uma profundidade máxima segura.
- **Processo sem Tokio**: Se o processo debugado não usa Tokio, o comando de tasks deve informar claramente que nenhum runtime async foi detectado.
- **Cargo.toml ausente ou inválido**: Se a integração Cargo é usada fora de um projeto Cargo, o sistema deve retornar um erro descritivo.
- **Binário desatualizado**: Se o binário em `target/debug/` é mais antigo que o código-fonte, o sistema deve avisar ou recompilar automaticamente.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema DEVE fornecer pretty printers para tipos padrão Rust (`Option<T>`, `Vec<T>`, `String`, `HashMap<K,V>`, `BTreeMap<K,V>`, `Result<T,E>`) que exibam representações legíveis ao invés de layouts de memória brutos.
- **FR-002**: O pretty printer DEVE diferenciar corretamente `Option::Some(valor)` e `Option::None`, exibindo o valor contido ou indicando ausência de valor.
- **FR-003**: O pretty printer DEVE exibir `Vec<T>` como uma lista indexada de elementos, truncando a exibição após um limite configurável (padrão: 100 elementos) com indicação de truncamento.
- **FR-004**: O pretty printer DEVE exibir `String` como texto legível (UTF-8) em vez de ponteiro/len/capacidade.
- **FR-005**: O sistema DEVE detectar a presença de um runtime Tokio no processo debugado e, quando detectado, listar as tasks async ativas.
- **FR-006**: Para cada task Tokio detectada, o sistema DEVE exibir: ID da task, estado de execução (running, idle, sleeping, completed), e nome/identificador da função async associada (quando disponível via símbolos PDB).
- **FR-007**: O sistema DEVE fornecer uma interface para iniciar debug a partir de um projeto Cargo, resolvendo automaticamente o binário-alvo com base no `Cargo.toml`.
- **FR-008**: A integração Cargo DEVE permitir especificar o pacote (para workspaces), o profile (`debug` ou `release`), e features opcionais a serem ativadas.
- **FR-009**: O sistema DEVE aplicar pretty printers recursivamente para tipos aninhados, com um limite de profundidade configurável (padrão: 5 níveis) para evitar travamentos em estruturas cíclicas ou muito profundas.

### Key Entities

- **Pretty Printer**: Regra de visualização para um tipo Rust específico. Atributos: tipo alvo (e.g., `std::vec::Vec`), função de formatação, limite de profundidade/tamanho, prioridade de matching.
- **Async Task**: Representação de uma tarefa assíncrona em execução no runtime Tokio. Atributos: task ID, estado (running, idle, sleeping, completed), função async associada, thread do runtime onde está agendada.
- **Cargo Target**: Configuração de build de um projeto Cargo. Atributos: nome do pacote, tipo de target (bin/lib/test), profile (debug/release), features ativas, caminho do artefato compilado.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Desenvolvedores podem visualizar o conteúdo de um `Vec<T>` com até 100 elementos em menos de 1 segundo após parar em um breakpoint.
- **SC-002**: 95% dos tipos Rust padrão (`Option`, `Vec`, `String`, `HashMap`, `Result`) encontrados em código de produção são exibidos com pretty printers legíveis sem configuração manual.
- **SC-003**: Desenvolvedores podem listar todas as tasks Tokio ativas em um processo debugado em menos de 2 segundos.
- **SC-004**: O tempo para iniciar uma sessão de debug a partir de um projeto Cargo é reduzido em 50% comparado à configuração manual do caminho do binário.
- **SC-005**: Pretty printers suportam aninhamento de tipos até 5 níveis de profundidade sem degradação perceptível de performance.

## Assumptions

- A aplicação debugada é compilada com informações de debug (PDB) e símbolos Rust, conforme as restrições da constituição.
- O runtime Tokio está presente no processo debugado quando a feature de async tasks é utilizada; aplicações sem Tokio recebem uma mensagem informativa.
- O usuário tem o Cargo instalado e configurado no ambiente para a integração Cargo funcionar.
- Pretty printers no MVP cobrem apenas tipos do `std`/`core` do Rust e informações do runtime Tokio; crates de terceiros podem ser adicionados em versões futuras.
- O formato de símbolos é PDB (Windows x86-64), conforme definido na constituição do projeto.
