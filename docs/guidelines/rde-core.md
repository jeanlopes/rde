# rde-core

Core debugger engine: traits, types, state machine, and event channels.

---

## Overview

`rde-core` is the heart of RDE. It defines:
- The `DebugEngine` state machine
- `DebugBackend` trait (abstracts Win32 / mock / future backends)
- Command and event protocols
- Shared types (registers, threads, modules, breakpoints)

## Architecture

```
REPL/UI ──► EngineCommand ──► DebugEngine ──► DebugLoopCommand ──► DebugBackend
                ▲                                              │
                └────── EngineEvent ◄──────────────────────────┘
```

## Key Types

### DebugEngine

```rust
pub struct DebugEngine<B: DebugBackend> {
    backend: B,
    session: Option<Session>,
    breakpoints: BreakpointManager,
    command_rx: mpsc::UnboundedReceiver<EngineCommand>,
    event_tx: mpsc::UnboundedSender<EngineEvent>,
    // ...
}
```

### Creating an Engine

```rust
use rde_core::DebugEngine;
use rde_win32::WindowsBackend;

let backend = WindowsBackend::new();
let (mut engine, command_tx, event_rx) = DebugEngine::new(backend);

// Run the engine in a background task
let handle = tokio::spawn(async move {
    engine.run().await.unwrap();
});
```

### Sending Commands

```rust
use rde_core::EngineCommand;

command_tx.send(EngineCommand::Launch {
    path: std::path::PathBuf::from("target/debug/my_app.exe"),
    args: vec![],
}).unwrap();
```

### Receiving Events

```rust
use rde_core::EngineEvent;

while let Some(event) = event_rx.recv().await {
    match event {
        EngineEvent::ProcessLaunched { pid } => println!("Launched: {pid}"),
        EngineEvent::BreakpointHit { kind, address, thread_id } => {
            // Handle breakpoint
        }
        EngineEvent::PrettyValue { value } => {
            // Handle pretty-printed value
        }
        EngineEvent::TaskList { tasks } => {
            // Handle task list
        }
        EngineEvent::MemoryBytes { address, bytes } => {
            // Handle raw memory bytes
        }
        _ => {}
    }
}
```

## EngineCommand

Commands sent from UI/REPL to the engine:

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
    Print { frame_id: u64, expression: String },
    ListTasks,
    CargoLaunch { manifest_path: PathBuf, package: Option<String>, target: Option<String>, profile: String, features: Vec<String> },
    SelectThread { id: ThreadId },
    Disassemble { address: Option<u64>, symbol: Option<String>, thread_id: Option<ThreadId>, count: Option<usize> },
    SetDisassemblyConfig { auto_show: Option<bool>, count: Option<usize> },
    Quit,
}
```

## EngineEvent

Events sent from engine to UI/REPL:

```rust
pub enum EngineEvent {
    ProcessLaunched { pid: u32 },
    ProcessAttached { pid: u32 },
    BreakpointHit { kind: BreakpointKind, address: u64, thread_id: ThreadId },
    SingleStep { address: u64, thread_id: ThreadId },
    ProcessExited { code: u32 },
    ThreadCreated { id: ThreadId, handle: usize },
    ThreadExited { id: ThreadId, code: u32 },
    ModuleLoaded { name: String, base: u64 },
    ModuleUnloaded { base: u64 },
    Exception { code: u32, address: u64 },
    Error { message: String },
    Output { message: String },
    Registers { ctx: RegisterContext },
    Disassembly { lines: Vec<DisassemblyLine> },
    BreakpointList { list: Vec<Breakpoint> },
    StackTrace { frames: Vec<StackFrame> },
    PrettyValue { value: PrettyValue },
    TaskList { tasks: Vec<AsyncTask> },
    CargoLaunchResult { result: Result<u32, String> },
    MemoryBytes { address: u64, bytes: Vec<u8> },
}
```

## DebugBackend Trait

```rust
#[async_trait]
pub trait DebugBackend: Send + Sync {
    async fn launch(&self, path: &std::path::Path, args: &[String]) -> Result<(ProcessHandle, DebugChannels), DebugError>;
    async fn attach(&self, pid: u32) -> Result<(ProcessHandle, DebugChannels), DebugError>;
    async fn continue_execution(&self, handle: &ProcessHandle) -> Result<(), DebugError>;
    async fn single_step(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError>;
    async fn read_memory(&self, handle: &ProcessHandle, address: u64, size: usize) -> Result<Vec<u8>, DebugError>;
    async fn write_memory(&self, handle: &ProcessHandle, address: u64, bytes: &[u8]) -> Result<(), DebugError>;
    async fn get_registers(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<RegisterContext, DebugError>;
    async fn set_registers(&self, handle: &ProcessHandle, thread_id: ThreadId, ctx: &RegisterContext) -> Result<(), DebugError>;
    async fn suspend_thread(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError>;
    async fn resume_thread(&self, handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError>;
}
```

## Shared Types

### RegisterContext

```rust
pub struct RegisterContext {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub rsp: u64,
    pub rip: u64, pub rflags: u64,
    pub r8: u64, pub r9: u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
}
```

### MemoryReader Trait

```rust
pub trait MemoryReader: Send + Sync {
    fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>, DebugError>;
}
```

### PrettyValue

```rust
pub enum PrettyValue {
    Scalar(String),
    Enum { name: String, payload: Option<Box<PrettyValue>> },
    Sequence(Vec<PrettyValue>),
    Map(Vec<(PrettyValue, PrettyValue)>),
    Raw(String),
    Truncated,
}
```

### AsyncTask & TaskState

```rust
pub struct AsyncTask {
    pub task_id: u64,
    pub state: TaskState,
    pub function_name: Option<String>,
    pub runtime_thread_id: Option<u32>,
}

pub enum TaskState {
    Running,
    Idle,
    Sleeping,
    Completed,
}
```

### CargoTarget

```rust
pub struct CargoTarget {
    pub package_name: String,
    pub target_name: String,
    pub target_kind: CargoTargetKind,
    pub profile: String,
    pub features: Vec<String>,
    pub artifact_path: PathBuf,
}

pub enum CargoTargetKind {
    Bin, Lib, Test, Bench, Example,
}
```

## Error Types

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
