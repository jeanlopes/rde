# Contract: Debug Engine Public API

**Feature**: 001-debugger-engine
**Date**: 2026-05-28

---

## Overview

O crate `rde-core` expõe uma API pública async para orquestrar sessões de debug. Outros crates
(`rde-cli`, futuros frontends) consomem esta API.

---

## Core Trait: `DebugBackend`

```rust
#[async_trait]
pub trait DebugBackend: Send + Sync {
    async fn launch(&self, path: &Path, args: &[String]) -> Result<ProcessHandle>;
    async fn attach(&self, pid: u32) -> Result<ProcessHandle>;
    async fn continue_execution(&self, handle: &ProcessHandle) -> Result<()>;
    async fn single_step(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<()>;
    async fn read_memory(&self, handle: &ProcessHandle, address: u64, size: usize) -> Result<Vec<u8>>;
    async fn write_memory(&self, handle: &ProcessHandle, address: u64, bytes: &[u8]) -> Result<()>;
    async fn get_registers(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<RegisterContext>;
    async fn set_registers(&self, handle: &ProcessHandle, thread_id: ThreadId, ctx: &RegisterContext) -> Result<()>;
    async fn suspend_thread(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<()>;
    async fn resume_thread(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<()>;
}
```

**Invariants**:
- `launch` e `attach` são mutuamente exclusivos por sessão.
- `continue_execution` só é válido quando o processo está pausado.
- `single_step` requer que a thread especificada esteja suspensa.
- `write_memory` para regiões executáveis deve ser precedido por `suspend_thread`.

---

## Engine Orchestrator: `DebugEngine`

```rust
pub struct DebugEngine<B: DebugBackend> {
    backend: B,
    session: Option<DebugSession>,
    command_rx: mpsc::Receiver<EngineCommand>,
    event_tx: mpsc::Sender<EngineEvent>,
}

impl<B: DebugBackend> DebugEngine<B> {
    pub fn new(backend: B) -> (Self, mpsc::Sender<EngineCommand>, mpsc::Receiver<EngineEvent>);
    pub async fn run(&mut self) -> Result<()>;
    pub async fn handle_command(&mut self, cmd: EngineCommand) -> Result<()>;
}
```

---

## Command/Event Types

### EngineCommand (REPL → Engine)

```rust
pub enum EngineCommand {
    Launch { path: PathBuf, args: Vec<String> },
    Attach { pid: u32 },
    Continue,
    StepInto,
    SetBreakpoint { address: Option<u64>, symbol: Option<String> },
    DeleteBreakpoint { id: BreakpointId },
    ReadRegisters { thread_id: Option<ThreadId> },
    ReadMemory { address: u64, size: usize },
    WriteMemory { address: u64, bytes: Vec<u8> },
    Backtrace { thread_id: Option<ThreadId> },
    ListThreads,
    ListModules,
    Quit,
}
```

### EngineEvent (Engine → REPL)

```rust
pub enum EngineEvent {
    ProcessLaunched { pid: u32 },
    ProcessAttached { pid: u32 },
    BreakpointHit { id: BreakpointId, address: u64, thread_id: ThreadId },
    SingleStep { address: u64, thread_id: ThreadId },
    ProcessExited { code: u32 },
    ThreadCreated { id: ThreadId },
    ThreadExited { id: ThreadId, code: u32 },
    ModuleLoaded { name: String, base: u64 },
    Exception { code: u32, address: u64 },
    Error { message: String },
    Output { message: String },
}
```

---

## Lifecycle

```
1. Engine::new(backend) -> (engine, cmd_tx, evt_rx)
2. Spawn debug_loop thread: backend.wait_event() loop
3. REPL sends commands via cmd_tx
4. Engine processes commands, mutates session state
5. Engine sends events via evt_tx
6. REPL receives events via evt_rx and displays them
7. Quit command -> engine teardown -> close handles -> exit
```

---

## Error Handling

Erros são propagados via `Result<T, DebugError>` na API e via `EngineEvent::Error` para o REPL.

```rust
pub enum DebugError {
    ProcessNotFound { pid: u32 },
    AccessDenied { pid: u32 },
    InvalidAddress { address: u64 },
    BreakpointNotFound { id: BreakpointId },
    SessionNotActive,
    AlreadyRunning,
    NotPaused,
    Win32Error { code: u32, message: String },
    Internal(String),
}
```

---

## Versioning

A API pública de `rde-core` segue semver. Alterações breaking em `DebugBackend`,
`EngineCommand`, ou `EngineEvent` exigem bump de versão MAJOR do crate.
