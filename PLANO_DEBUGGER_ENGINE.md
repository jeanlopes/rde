# Plano de Implementação: Rust Debugger Engine (RDE)

## Visão Geral

Criar uma **biblioteca de debugger engine em Rust**, para **Windows**, inspirada na arquitetura de debuggers C open source (x64dbg/TitanEngine), que supere as limitações do LLDB no ecossistema Windows+Rust.

**Objetivo central:** Debugger nativo Windows, REPL-first, com injeção de breakpoints em runtime sem congelar a UI, arquitetura 100% Rust, sem FFI para LLDB/GDB.

---

## Por que não LLDB?

| Problema LLDB | Solução RDE |
|---------------|-------------|
| Arquitetura bloqueante — UI congela durante execução | Event loop async + message passing |
| Não permite adicionar breakpoints via UI durante execução | Runtime breakpoint injection via command channel |
| Windows é cidadão de segunda classe | Windows-first, API nativa |
| PDB parsing problemático | DbgHelp nativo do Windows |
| Código legado monstruoso | Código Rust moderno, modular, testável |
| FFI frágil (lldb-sys) | Zero FFI — syscalls Windows via `windows-rs` |

---

## Referência em C: TitanEngine / x64dbg

**Referência primária:** [TitanEngine](https://github.com/x64dbg/TitanEngine)

**Por que TitanEngine:**
- Engine de debugger Windows user-mode
- Abstrai `WaitForDebugEvent`, `ReadProcessMemory`, `WriteProcessMemory`
- Arquitetura compreensível (~15k linhas C++)
- Foco em x64 Windows (igual seu alvo)
- Licença GPL (referência, não cópia)

**Referência secundária:** [x64dbg](https://github.com/x64dbg/x64dbg)
- Frontend que consome TitanEngine
- Mostra como separar Engine de UI

---

## Stack Tecnológico

| Camada | Crate/Tool |
|--------|-----------|
| Win32 API | `windows` (oficial Microsoft) |
| Disassembly | `capstone` (Capstone Engine bindings Rust) |
| Symbols/PDB | `DbgHelp.dll` via `windows` crate |
| Async/Channels | `tokio::sync::mpsc` ou `crossbeam-channel` |
| TUI (futuro) | `ratatui` |
| Parser REPL | `chumsky` ou `winnow` |
| Serialização | `serde` (para config, snapshots) |
| Logging | `tracing` |
| Testes | `insta`, `mockall` |

---

## Arquitetura

```
┌─────────────────────────────────────────────┐
│  CLI / REPL / TUI / DAP (Futuro)            │
│  - Parser de comandos                         │
│  - Auto-complete                              │
│  - Renderização                               │
└──────────────┬──────────────────────────────┘
               │ commands via channel
               ▼
┌─────────────────────────────────────────────┐
│  Debug Engine (Core) — Biblioteca           │
│  - Estado do debugger                         │
│  - Fila de comandos (mpsc)                    │
│  - Orquestração                               │
└──────────────┬──────────────────────────────┘
               │
       ┌───────┴───────┐
       ▼               ▼
┌─────────────┐  ┌──────────────┐
│ Debug Loop  │  │ Symbol Engine│
│ Thread      │  │ (DbgHelp)    │
└──────┬──────┘  └──────────────┘
       │
       ▼
┌─────────────────────────────────────────────┐
│  Windows Backend                            │
│  - CreateProcessW / DebugActiveProcess        │
│  - WaitForDebugEventEx                        │
│  - Read/WriteProcessMemory                    │
│  - Get/SetThreadContext                       │
│  - Suspend/ResumeThread                       │
└─────────────────────────────────────────────┘
```

---

## Estrutura de Diretórios

```
rde/
├── Cargo.toml
├── crates/
│   ├── rde-core/           # Traits, tipos, estado
│   ├── rde-win32/          # Backend Windows (Win32 Debug API)
│   ├── rde-symbols/        # PDB/DbgHelp integration
│   ├── rde-breakpoint/     # Gerenciamento de breakpoints
│   ├── rde-repl/           # Parser + executor de comandos
│   └── rde-cli/            # Binário CLI (depende de todos)
├── docs/
│   └── win32-debug-api.md
├── examples/
│   └── hello_debuggee.rs
└── tests/
    └── integration_tests.rs
```

---

## Fases de Implementação

### FASE 0 — Setup e Fundação (Dias 1-3)

**Objetivo:** Compilar, testar infraestrutura, entender APIs.

- [ ] Criar workspace Cargo com `crates/` separados
- [ ] Configurar `windows` crate com features necessárias:
  ```toml
  features = [
    "Win32_System_Diagnostics_Debug",
    "Win32_System_Threading",
    "Win32_System_Memory",
    "Win32_Foundation",
    "Win32_Security",
  ]
  ```
- [ ] Criar `examples/hello_debuggee.rs` — programa simples para debugar
- [ ] Criar doc `docs/win32-debug-api.md` mapeando cada API usada
- [ ] Teste mínimo: spawn processo com `DEBUG_ONLY_THIS_PROCESS` e printar eventos

**APIs foco:** `CreateProcessW`, `WaitForDebugEventEx`, `ContinueDebugEvent`

---

### FASE 1 — Debug Loop + Controle Básico (Dias 4-10)

**Objetivo:** Ter um loop de debug funcional com continue/step.

**Implementar em `rde-win32`:**

```rust
pub struct WindowsBackend {
    process: HANDLE,
    main_thread: HANDLE,
    // ...
}

impl WindowsBackend {
    pub fn launch(path: &Path) -> Result<Self>;
    pub fn attach(pid: u32) -> Result<Self>;
    pub fn wait_event(&self) -> Result<DebugEvent>;
    pub fn continue_event(&self, event: &DebugEvent, status: ContinueStatus);
    pub fn single_step(&self, thread_id: u32) -> Result<()>;
}
```

**Event loop thread (em `rde-core`):**

```rust
loop {
    let event = backend.wait_event()?;
    
    // Notifica engine via channel (non-blocking)
    event_tx.send(event.clone())?;
    
    // Verifica se há comando pendente (timeout curto)
    if let Ok(cmd) = command_rx.try_recv() {
        engine.handle_command(cmd);
    }
    
    backend.continue_event(&event, ContinueStatus::Handled);
}
```

- [ ] Mapear todos `DEBUG_EVENT_CODE`:
  - `EXCEPTION_DEBUG_EVENT` (breakpoints, access violation, etc.)
  - `CREATE_THREAD_DEBUG_EVENT`
  - `CREATE_PROCESS_DEBUG_EVENT`
  - `EXIT_THREAD_DEBUG_EVENT`
  - `EXIT_PROCESS_DEBUG_EVENT`
  - `LOAD_DLL_DEBUG_EVENT`
  - `OUTPUT_DEBUG_STRING_EVENT`
  - `RIP_EVENT`
- [ ] Implementar `ContinueDebugEvent` com `DBG_CONTINUE` vs `DBG_EXCEPTION_NOT_HANDLED`
- [ ] Implementar single-step via `SetThreadContext` (flag Trap/EF)
- [ ] Testes: lançar hello_debuggee, capturar eventos, continuar, sair

---

### FASE 2 — Breakpoints em Runtime (Dias 11-18)

**Objetivo:** Breakpoints funcionais, adicionáveis/removíveis a qualquer momento.

**Por que é possível:** `WriteProcessMemory` funciona enquanto processo roda. Basta coordenar com thread suspension se necessário.

**Estrutura:**

```rust
// rde-breakpoint/src/lib.rs
pub struct BreakpointManager {
    breakpoints: HashMap<u64, Breakpoint>,
}

pub struct Breakpoint {
    address: u64,
    original_byte: u8,  // salvo antes de escrever 0xCC
    enabled: bool,
    hit_count: u64,
}

impl BreakpointManager {
    pub fn set(&mut self, addr: u64, backend: &dyn MemoryBackend) -> Result<()>;
    pub fn remove(&mut self, addr: u64, backend: &dyn MemoryBackend) -> Result<()>;
    pub fn handle_hit(&mut self, addr: u64, backend: &dyn MemoryBackend) -> Result<HitAction>;
}
```

**Fluxo de hit:**
1. Exception `STATUS_BREAKPOINT` (0x80000003) chega
2. Engine identifica endereço do breakpoint (RIP-1)
3. Restaura byte original com `WriteProcessMemory`
4. Decrementa RIP (para re-executar instrução original)
5. Seta flag Trap no EFLAGS/RFLAGS para single-step
6. `ContinueDebugEvent`
7. No próximo evento (single-step), reinstala 0xCC
8. `ContinueDebugEvent` normal

**Runtime injection:**
- REPL envia `Command::SetBreakpoint(addr)` via channel
- Engine recebe, chama `BreakpointManager::set()`
- Se processo está rodando: `SuspendThread` → write → `ResumeThread`
- Sem bloquear REPL!

- [ ] Implementar `ReadProcessMemory` / `WriteProcessMemory`
- [ ] Implementar `GetThreadContext` / `SetThreadContext`
- [ ] Implementar breakpoint com INT3 (0xCC)
- [ ] Implementar "restore + step + reinstall" pattern
- [ ] Implementar `SuspendThread` / `ResumeThread` para safe write
- [ ] Teste: adicionar breakpoint durante execução via channel

---

### FASE 3 — REPL Embutido (Dias 19-25)

**Objetivo:** Interface interativa funcional, comandos parseáveis.

**Arquitetura REPL:**

```
Thread REPL
    ↓ (input do usuário)
Parser → Command enum
    ↓
mpsc::channel → Engine
    ↓
Feedback via outro channel → Print no terminal
```

**Command enum:**

```rust
pub enum Command {
    Launch(PathBuf),
    Attach(u32),
    Continue,
    StepInto,
    StepOver,      // (futuro, precisa de disassembly)
    Breakpoint { address: Option<u64>, symbol: Option<String> },
    DeleteBreakpoint(u64),
    Registers,
    ReadMemory { address: u64, size: usize },
    WriteMemory { address: u64, bytes: Vec<u8> },
    Backtrace,
    Threads,
    Modules,
    Quit,
}
```

**Parser:** Inicialmente `split_whitespace` + match. Futuro: `chumsky`.

**REPL loop:**

```rust
loop {
    print!("rde> ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match parse_command(&input) {
        Ok(cmd) => command_tx.send(cmd)?,
        Err(e) => eprintln!("Erro: {}", e),
    }
}
```

- [ ] Implementar parser de comandos
- [ ] Implementar channel bidirecional (Command in, Event out)
- [ ] Integrar REPL com Debug Engine
- [ ] Comandos: `break`, `continue`, `step`, `regs`, `x` (examine memory), `quit`
- [ ] Teste: sessão completa de debug interativa

---

### FASE 4 — Registradores e Memória (Dias 26-32)

**Objetivo:** Inspeção completa de estado.

**Registradores x64:**

```rust
#[repr(C)]
pub struct RegisterContext {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub rip: u64,
    pub r8: u64, pub r9: u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    pub eflags: u32,
    pub cs: u16, pub ds: u16, pub es: u16, pub fs: u16, pub gs: u16, pub ss: u16,
    pub mx_csr: u32,
    pub fpu: [u8; 512], // SSE/AVX futuro
}
```

**Memória:**
- `ReadProcessMemory` com pretty print (hex + ASCII)
- `WriteProcessMemory` para patches
- `VirtualQueryEx` para mapear regiões de memória

- [ ] Mapear `WOW64_CONTEXT` / `CONTEXT` para x64
- [ ] Implementar `registers` command com formatação
- [ ] Implementar `x/<count><format>` (estilo GDB)
- [ ] Implementar `set <reg> = <value>`
- [ ] Implementar `info mem` com VirtualQueryEx
- [ ] Teste: inspecionar registradores em breakpoint, modificar RIP

---

### FASE 5 — Símbolos e Stack Trace (Dias 33-42)

**Objetivo:** Resolver nomes de funções, linhas de código, stack traces.

**DbgHelp APIs:**

```rust
// Carregadas dinamicamente via LoadLibrary/GetProcAddress
// ou via windows crate se disponível
SymInitialize
SymFromAddr
SymGetLineFromAddr64
StackWalk64
SymLoadModuleEx
```

**Fluxo:**
1. Processo é criado → `SymInitialize(process, search_path, true)`
2. DLL carregada → `SymLoadModuleEx`
3. Breakpoint em endereço → `SymFromAddr` → nome da função
4. Stack trace → `StackWalk64` + `SymFromAddr` em cada frame

**Demangling Rust:**
- `rustc_demangle` crate para nomes de funções Rust

```rust
pub struct SymbolInfo {
    pub name: String,
    pub module: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub address: u64,
}

pub struct StackFrame {
    pub address: u64,
    pub symbol: Option<SymbolInfo>,
    pub return_address: u64,
    pub frame_pointer: u64,
    pub stack_pointer: u64,
}
```

- [ ] Integrar DbgHelp.dll via `libloading` ou bindings
- [ ] Implementar `SymInitialize`/`SymCleanup`
- [ ] Implementar resolução de símbolos por endereço
- [ ] Implementar stack walker (`StackWalk64`)
- [ ] Adicionar breakpoint por nome de função (`break main`)
- [ ] Implementar `rustc_demangle`
- [ ] Teste: stack trace em hello_debuggee com símbolos Rust

---

### FASE 6 — Threads e Módulos (Dias 43-48)

**Objetivo:** Multi-threading e inspeção de DLLs.

- [ ] Rastrear threads: `CREATE_THREAD_DEBUG_EVENT`, `EXIT_THREAD_DEBUG_EVENT`
- [ ] Implementar `info threads`
- [ ] Implementar `thread <id>` para switch de contexto
- [ ] Rastrear módulos: `LOAD_DLL_DEBUG_EVENT`, `UNLOAD_DLL_DEBUG_EVENT`
- [ ] Implementar `info modules`
- [ ] Teste: debuggee com threads, ver criação/encerramento

---

### FASE 7 — Disassembly (Dias 49-55)

**Objetivo:** Ver código assembly em tempo real.

**Capstone Engine:**

```rust
use capstone::prelude::*;

let cs = Capstone::new()
    .x86()
    .mode(arch::x86::ArchMode::Mode64)
    .build()?;

let instructions = cs.disasm_all(code, address)?;
```

- [ ] Integrar `capstone` crate
- [ ] Implementar `disassemble <address> [count]`
- [ ] Mostrar instruções com endereços, bytes, mnemônicos, operandos
- [ ] Destacar breakpoint atual na disassembly
- [ ] Teste: disassembly de função em breakpoint

---

### FASE 8 — Polish e TUI (Dias 56-65)

**Objetivo:** Experiência de usuário profissional.

- [ ] Integrar `ratatui` com layout:
  ```
  ┌──────────────┬─────────────┐
  │ Source/Asm   │ Registers   │
  ├──────────────┼─────────────┤
  │ Stack        │ Breakpoints │
  ├──────────────┴─────────────┤
  │ Command/REPL               │
  └────────────────────────────┘
  ```
- [ ] Syntax highlighting no REPL
- [ ] Auto-complete de comandos e símbolos
- [ ] Config file (`rde.toml`)
- [ ] Logging com `tracing`
- [ ] Documentação completa da API pública

---

### FASE 9 — Rust-Specific Features (Futuro)

**O diferencial real:**

- [ ] **Async-aware debugging:** Inspecionar `tokio` runtime tasks
- [ ] **Pretty printers:** `Option<T>`, `Result<T,E>`, `Vec<T>`, `String` visualizados elegantemente
- [ ] **Enum variant inspection:** Mostrar discriminante + dados
- [ ] **Trait object introspection:** Resolver vtables
- [ ] **Integration com `cargo`:** `cargo debug`, `cargo debug --example foo`
- [ ] **DAP (Debug Adapter Protocol):** Integração com VSCode

---

## Design Decisions Críticas

### 1. Message Passing vs Shared State

```rust
// Preferido — nunca bloqueia REPL
enum EngineCommand {
    SetBreakpoint(u64),
    // ...
}

enum EngineEvent {
    BreakpointHit { address: u64, thread_id: u32 },
    Output(String),
    // ...
}

// Channels:
// main (REPL) → engine: EngineCommand
// engine → main: EngineEvent
```

### 2. Thread Model

| Thread | Função |
|--------|--------|
| Main | REPL input, renderização |
| Debug Loop | `WaitForDebugEventEx` exclusivo |
| Symbol Worker | Parse PDB, resolve símbolos (pesado, não bloqueia) |
| TUI Render | `ratatui` loop (60fps) |

### 3. Ergonomia de API Pública

```rust
use rde_core::{Debugger, Command, Event};

#[tokio::main]
async fn main() -> Result<()> {
    let mut dbg = Debugger::launch("target/debug/myapp.exe").await?;
    
    dbg.set_breakpoint("main").await?;
    dbg.continue_execution().await?;
    
    while let Some(event) = dbg.next_event().await {
        match event {
            Event::BreakpointHit { address, .. } => {
                println!("Hit at 0x{:x}", address);
                let regs = dbg.registers().await?;
                println!("RIP: 0x{:x}", regs.rip);
                dbg.step_into().await?;
            }
            Event::ProcessExited { code } => break,
            _ => {}
        }
    }
    
    Ok(())
}
```

---

## Testes

### Estratégia

- **Unit tests:** `rde-core` state machines, parser
- **Integration tests:** Debugar `examples/hello_debuggee.rs`
- **Mock backend:** `MockDebugBackend` implementando traits para testar engine sem Windows
- **Snapshot tests:** Output de comandos com `insta`

### CI

```yaml
# .github/workflows/test.yml
runs-on: windows-latest
steps:
  - uses: actions/checkout@v4
  - uses: dtolnay/rust@stable
  - run: cargo test --workspace
  - run: cargo test --example hello_debuggee
```

---

## Riscos e Mitigações

| Risco | Mitigação |
|-------|-----------|
| DbgHelp não tem bindings oficiais Rust | Usar `libloading` + struct de funções, ou `bindgen` em cabeçalhos |
| Deadlock no debug loop | Nunca fazer I/O bloqueante na thread de `WaitForDebugEvent` |
| Race condition em breakpoint write | Sempre `SuspendThread` antes de `WriteProcessMemory` no endereço ativo |
| Symbol loading lento | Fazer em thread separada, cachear resultados |
| x64dbg/TitanEngine C++ complexo | Focar em entender o fluxo, não copiar linha a linha |

---

## Métricas de Sucesso

- [ ] Consegue debugar `examples/hello_debuggee.rs` do início ao fim
- [ ] Breakpoint adicionado durante `continue` funciona sem travar REPL
- [ ] Stack trace mostra nomes de funções Rust demangled
- [ ] REPL responde em < 50ms mesmo com processo rodando
- [ ] Zero dependências de C++ (LLVM, LLDB, GDB)

---

## Próximos Passos Imediatos

1. Criar workspace Cargo (`cargo new --lib rde`)
2. Adicionar `windows` crate com features corretas
3. Escrever `examples/hello_debuggee.rs`
4. Implementar FASE 0: spawn com `DEBUG_ONLY_THIS_PROCESS` + print de eventos
5. Testar no terminal: `cargo run --example hello_debuggee`

---

*Plano criado em 2026-05-28. Revisar a cada fase concluída.*
