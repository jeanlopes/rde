# Data Model: Rust Debugger Engine (RDE)

**Feature**: 001-debugger-engine
**Date**: 2026-05-28

---

## Core Entities

### DebugSession

Representa uma sessão ativa de debugging.

```rust
pub struct DebugSession {
    pub id: SessionId,              // UUID v4
    pub target: Target,             // Programa alvo (path ou PID)
    pub state: SessionState,        // Running | Paused | Exited
    pub breakpoints: BreakpointManager,
    pub threads: HashMap<ThreadId, Thread>,
    pub modules: HashMap<ModuleBase, Module>,
    pub selected_thread: ThreadId,  // Thread atualmente em foco
}

pub enum SessionState {
    Running,
    Paused { reason: PauseReason },
    Exited { code: u32 },
}

pub enum PauseReason {
    BreakpointHit { address: u64, breakpoint_id: BreakpointId },
    SingleStepComplete { address: u64 },
    Exception { code: u32, address: u64 },
    ProcessExited,
}
```

**Validation rules**:
- `selected_thread` MUST exist in `threads` when `state` is `Paused`.
- `state` transitions: `Running` → `Paused` (via event), `Paused` → `Running` (via Continue),
  any → `Exited` (on process termination).

---

### Target

Identifica o programa a ser debugado.

```rust
pub enum Target {
    Launch { path: PathBuf, args: Vec<String>, working_dir: Option<PathBuf> },
    Attach { pid: u32 },
}
```

**Validation rules**:
- `Launch.path` MUST point to an existing executable file.
- `Attach.pid` MUST reference a running process accessible with debug privileges.

---

### Breakpoint

Representa um ponto de parada.

```rust
pub struct Breakpoint {
    pub id: BreakpointId,           // u64, auto-increment
    pub address: u64,               // Endereço virtual no espaço do alvo
    pub original_byte: u8,          // Byte salvo antes de escrever 0xCC
    pub state: BreakpointState,
    pub hit_count: u64,
}

pub enum BreakpointState {
    Enabled,
    Disabled,
    Pending,                       // Endereço ainda não resolvido (símbolo não carregado)
}
```

**Validation rules**:
- `address` MUST be aligned to a valid code page (executable permission).
- `original_byte` MUST be preserved accurately for restoration on hit.
- A breakpoint at the same `address` MUST NOT be duplicated.

---

### Thread

Representa uma thread do processo alvo.

```rust
pub struct Thread {
    pub id: ThreadId,               // OS thread ID (u32)
    pub handle: RawHandle,          // HANDLE opaco (wrapping em rde-win32)
    pub state: ThreadState,
    pub context: Option<RegisterContext>, // Preenchido apenas quando pausada
}

pub enum ThreadState {
    Running,
    Suspended,
    Exited { exit_code: u32 },
}
```

**Validation rules**:
- `context` is `Some` ONLY when `state` is `Suspended` or the session is `Paused`.
- `handle` MUST be closed when the thread exits or the session ends.

---

### RegisterContext

Estado dos registradores x86-64.

```rust
pub struct RegisterContext {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub rsp: u64,
    pub rip: u64,
    pub r8: u64, pub r9: u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    pub rflags: u64,
    // Segment registers e FPU omitidos do MVP podem ser adicionados posteriormente
}
```

---

### Module

DLL ou executável carregado no espaço de endereço do alvo.

```rust
pub struct Module {
    pub name: String,               // Nome do arquivo (e.g., "kernel32.dll")
    pub base_address: u64,          // Endereço base de carregamento
    pub size: u64,                  // Tamanho em bytes
    pub path: Option<PathBuf>,      // Caminho completo no disco
    pub symbols_loaded: bool,       // DbgHelp carregou símbolos?
}
```

---

### StackFrame

Frame de uma pilha de chamadas.

```rust
pub struct StackFrame {
    pub frame_number: u32,
    pub return_address: u64,
    pub frame_pointer: u64,
    pub stack_pointer: u64,
    pub symbol: Option<SymbolInfo>,
}

pub struct SymbolInfo {
    pub name: String,               // Nome demangled (se Rust) ou raw
    pub module: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub address: u64,
}
```

---

## State Transitions

### SessionState Machine

```
[Created]
   |
   v
[Running] <---> [Paused]
   |                |
   +----------------+
   |
   v
[Exited]
```

**Transitions**:
- `Created` → `Running`: Após `CreateProcessW` + `WaitForDebugEvent` (CREATE_PROCESS_DEBUG_EVENT)
- `Running` → `Paused`: Recebimento de `EXCEPTION_DEBUG_EVENT` (breakpoint) ou `OUTPUT_DEBUG_STRING_EVENT`
- `Paused` → `Running`: Comando `Continue` enviado via channel
- `Running` → `Exited`: `EXIT_PROCESS_DEBUG_EVENT`
- `Paused` → `Exited`: `EXIT_PROCESS_DEBUG_EVENT` (raro, mas possível se thread principal morre)

### BreakpointState Machine

```
[Pending] --(símbolo resolvido)--> [Enabled]
   |
   v
[Enabled] <-> [Disabled]
```

---

## Relationships

```
DebugSession *--1 Target
DebugSession *--* Breakpoint
DebugSession *--* Thread
DebugSession *--* Module
Thread 1--0..1 RegisterContext
StackFrame *--0..1 SymbolInfo
```

---

## Invariants

1. **Thread-Context Consistency**: Se `SessionState` é `Paused`, pelo menos uma thread existe
   com `context = Some`.
2. **Breakpoint Uniqueness**: Não podem existir dois breakpoints com o mesmo `address` na
   mesma sessão.
3. **Selected Thread Validity**: `DebugSession.selected_thread` deve sempre referenciar uma
   thread existente em `DebugSession.threads`.
4. **Handle Lifecycle**: Todos os handles de processo e thread (do Win32) devem ser fechados
   quando a sessão termina ou a entidade é removida.
