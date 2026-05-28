# Data Model: Thread and Module Tracking

**Feature**: 002-thread-module-tracking
**Date**: 2026-05-28

---

## Entity: Thread

```rust
pub struct Thread {
    /// OS thread identifier (TID).
    pub id: ThreadId,

    /// Cached handle from CREATE_THREAD_DEBUG_EVENT.
    /// Closed on EXIT_THREAD_DEBUG_EVENT or session cleanup.
    pub handle: RawHandle,

    /// Current lifecycle state.
    pub state: ThreadState,

    /// CPU register snapshot (populated when thread is paused).
    pub context: Option<RegisterContext>,
}
```

### State Transitions

```text
[Created] --(CREATE_THREAD_DEBUG_EVENT)--> Running
Running --(thread suspended by debugger)--> Suspended
Suspended --(thread resumed)--> Running
Running --(EXIT_THREAD_DEBUG_EVENT)--> Exited
Suspended --(EXIT_THREAD_DEBUG_EVENT)--> Exited
```

### Validation Rules
- `id` must be unique within the session.
- `handle` must be valid (non-zero) while state is not `Exited`.
- `context` is `Some` only when the thread is paused (debugger has intervened).

---

## Entity: Module

```rust
pub struct Module {
    /// File name (e.g., "kernel32.dll").
    pub name: String,

    /// Load address in target process address space.
    pub base_address: u64,

    /// Size in bytes (from `MODULEINFO` or image headers).
    pub size: u64,

    /// Full filesystem path (when resolvable).
    pub path: Option<PathBuf>,

    /// Whether DbgHelp has loaded symbols for this module.
    pub symbols_loaded: bool,
}
```

### Validation Rules
- `base_address` must be unique within the session (two modules cannot load at the same base).
- `size` may be zero initially if `EnumProcessModules` fails; updated when available.
- `symbols_loaded` is `false` until `SymLoadModuleEx` succeeds.

---

## Entity: Session (Updated)

```rust
struct Session {
    handle: ProcessHandle,

    /// All threads known in the target process.
    threads: HashMap<ThreadId, Thread>,

    /// All modules loaded in the target process.
    modules: HashMap<u64, Module>,

    /// Thread used as default context for regs/bt/step.
    /// Updated automatically when the selected thread exits.
    selected_thread: Option<ThreadId>,
}
```

### Invariants
- `selected_thread`, if `Some`, must reference a `Thread` in `threads` whose state is not `Exited`.
- On `EXIT_THREAD_DEBUG_EVENT`, if the exited thread is `selected_thread`, the engine MUST
  reassign `selected_thread` to the oldest remaining non-exited thread, or `None` if empty.
- `threads` and `modules` are cleared on `ProcessExited`.

---

## Entity: DebugEngine (Updated State)

```rust
pub struct DebugEngine<B: DebugBackend> {
    backend: B,
    session: Option<Session>,
    breakpoints: BreakpointManager,
    command_rx: mpsc::UnboundedReceiver<EngineCommand>,
    event_tx: mpsc::UnboundedSender<EngineEvent>,
    debug_event_rx: mpsc::UnboundedReceiver<EngineEvent>,
    debug_loop_tx: Option<mpsc::UnboundedSender<DebugLoopCommand>>,
    stepping_over_breakpoint: Option<u64>,
}
```

### Changes from Spec 001
- `Session` now carries `threads`, `modules`, and `selected_thread`.
- `handle_command` for `StepInto`, `ReadRegisters`, and `Backtrace` uses
  `session.selected_thread` instead of hardcoded `0`.
- `handle_event` has explicit arms for `ThreadCreated`, `ThreadExited`, `ModuleLoaded`, and
  `ModuleUnloaded` instead of falling through to the catch-all `_ =>`.

---

## Event / Command Additions

### EngineCommand (new variant)
```rust
pub enum EngineCommand {
    // ... existing variants ...
    SelectThread {
        id: ThreadId,
    },
}
```

### EngineEvent (new variant)
```rust
pub enum EngineEvent {
    // ... existing variants ...
    ModuleUnloaded {
        base: u64,
    },
}
```
