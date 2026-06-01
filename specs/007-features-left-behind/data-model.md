# Data Model: Features Left Behind

**Phase**: 1 — Design
**Date**: 2026-06-01
**Feature**: [spec.md](spec.md) | [plan.md](plan.md) | [research.md](research.md)

---

## New Command Variants (`rde-core/src/lib.rs` — `EngineCommand` enum)

```rust
/// Advance one machine instruction in the current frame without entering callees.
StepOver,

/// Run to the end of the current function and stop at the caller.
StepOut,

/// Configure pretty-printer output limits.
SetPrintConfig {
    /// Maximum number of collection elements shown (None = unchanged).
    limit: Option<usize>,
    /// Maximum nesting depth for structs/enums (None = unchanged).
    depth: Option<usize>,
    /// Enable/disable pretty printing (None = unchanged).
    pretty: Option<bool>,
},
```

*Existing commands not changed: `StepInto`, `Continue`, `SetBreakpoint`, `DeleteBreakpoint`, `CargoLaunch`, `SelectThread`, etc.*

---

## New Event Variants (`rde-core/src/lib.rs` — `EngineEvent` enum)

```rust
/// Emitted after a StepOver or StepOut operation completes.
StepCompleted {
    thread_id: u32,
    /// Address the IP is now at (after the step).
    address: u64,
},
```

*`BreakpointHit` continues to be used for user-defined BP hits. `StepCompleted` is a distinct event so the REPL can display step confirmation without it looking like a breakpoint.*

---

## Modified Type: `BreakpointKind` (`rde-core/src/lib.rs` or `rde-core/src/events.rs`)

```rust
pub enum BreakpointKind {
    /// Windows ntdll initial breakpoint — never surface to user, always DBG_CONTINUE.
    SystemInitial,
    /// A breakpoint set by the user via `break <symbol|address>`.
    UserDefined(u32 /* bp id */),
    /// An internal temporary breakpoint planted by step-over or step-out logic.
    /// MUST NOT be surfaced as a hit event to the user.
    Temporary,
}
```

---

## New Engine Fields (`rde-core/src/engine.rs` — `DebugEngine<B>`)

```rust
/// Address of the temp BP planted by a StepOver command.
/// None when not stepping over.
stepping_over_breakpoint: Option<u64>,   // already exists

/// Address of the temp BP planted by a StepOut command.
/// None when not stepping out.
stepping_out_breakpoint: Option<u64>,    // NEW

/// Current pretty-printer configuration.
print_config: PrintConfig,               // NEW
```

---

## New Struct: `PrintConfig` (`rde-core/src/lib.rs`)

```rust
/// Runtime configuration for the pretty-printer.
#[derive(Debug, Clone)]
pub struct PrintConfig {
    /// Max collection elements to show. Default: 100.
    pub limit: usize,
    /// Max nesting depth to expand. Default: 5.
    pub depth: usize,
    /// Enable structured pretty output. Default: true.
    pub pretty: bool,
}

impl Default for PrintConfig {
    fn default() -> Self {
        Self { limit: 100, depth: 5, pretty: true }
    }
}
```

---

## New Module: `rde-win32/src/step.rs`

```rust
/// Plant a temporary breakpoint at `address` in the target process.
/// Returns the original byte at that address (must be saved by caller).
pub fn plant_temp_breakpoint(
    handle: &ProcessHandle,
    address: u64,
) -> Result<u8, DebugError>;

/// Restore a byte at `address` that was overwritten by a temp breakpoint.
pub fn restore_temp_breakpoint(
    handle: &ProcessHandle,
    address: u64,
    original_byte: u8,
) -> Result<(), DebugError>;

/// Compute the address for a step-over temp BP.
/// Decodes the instruction at `rip` using Capstone to get its byte length,
/// then returns `rip + length`.
pub fn next_instruction_address(
    handle: &ProcessHandle,
    rip: u64,
) -> Result<u64, DebugError>;

/// Read the return address for step-out from the top of the stack.
/// Returns `[RSP]` as a u64 on x64.
pub fn read_return_address(
    handle: &ProcessHandle,
    rsp: u64,
) -> Result<u64, DebugError>;
```

---

## REPL Command Parsing Changes (`rde-repl/src/parser.rs`)

| Input tokens | Parsed as | Notes |
|---|---|---|
| `next` / `n` | `EngineCommand::StepOver` | New |
| `finish` / `f` | `EngineCommand::StepOut` | New |
| `set print-limit <N>` | `SetPrintConfig { limit: Some(N), .. }` | New |
| `set print-depth <N>` | `SetPrintConfig { depth: Some(N), .. }` | New |
| `set pretty-print on\|off` | `SetPrintConfig { pretty: Some(bool), .. }` | New |
| `attach <pid>` | `EngineCommand::Attach { pid }` | Already parsed; verify wiring |
| `thread <id>` | `EngineCommand::SelectThread { id }` | Already parsed; verify wiring |

---

## Debuggee Additions (`examples/rust_app_example/src/main.rs`)

### New demo modes

| `--demo` value | Behavior |
|---|---|
| `threaded` | Spawns 3 worker threads each inserting 3 values into a shared `Mutex<BinaryTree>` |
| `panic` | Inserts 3 values then calls `panic!("test panic")` |
| `stress-test` | Inserts 100 values sequentially (existing behavior, confirm name) |

### Modified demo mode

| `--demo` value | Change |
|---|---|
| `insert-sequence` | Insert `[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]` to guarantee left- and right-rotations |

### New types (in scope at breakpoint on `insert`)

```rust
pub struct TreeStats {
    pub insertions: usize,
    pub rotations: usize,
    pub deletions: usize,
}

pub enum TreeError {
    DuplicateValue(i32),
    EmptyTree,
    NotFound(i32),
}
```

---

## State Transitions

### Step-Over State Machine

```
Paused(at user BP)
  → [user sends "next"]
  → SteppingOver { temp_bp_addr: RIP+L, saved_byte: u8 }
  → [EXCEPTION_BREAKPOINT at temp_bp_addr]
  → Paused(at RIP+L, StepCompleted event emitted)
```

### Step-Out State Machine

```
Paused(at user BP or step hit)
  → [user sends "finish"]
  → SteppingOut { temp_bp_addr: return_addr, saved_byte: u8 }
  → [EXCEPTION_BREAKPOINT at return_addr]
  → Paused(at return_addr, StepCompleted event emitted)
```

### Running-State Command Rejection (US4)

The engine must track whether the process is currently running (after `Continue` with no pending events) to reject commands that require the process to be paused:

```rust
enum ProcessState {
    Paused,   // Inside WaitForDebugEventEx handler; commands accepted
    Running,  // ContinueDebugEvent called; awaiting next event
}
```

Commands rejected when `ProcessState::Running`: `StepInto`, `StepOver`, `StepOut`, `ReadRegisters`, `ReadMemory`, `Backtrace`, `Print`.

Commands accepted when `ProcessState::Running`: `SetBreakpoint`, `DeleteBreakpoint`, `Quit`.

---

## Validation Rules

- `SetPrintConfig { limit: Some(0) }` — valid; means "show 0 items" (collapse all collections).
- `SelectThread { id }` with an exited thread — `DebugError::Internal("thread exited")`.
- `StepOver` / `StepOut` when `ProcessState::Running` — `DebugError::ProcessRunning`.
- `StepOut` when RSP is 0 or unreadable — `DebugError::Internal("invalid stack pointer")`.
- `plant_temp_breakpoint` at an address already in `BreakpointManager` — overwrite allowed; restore original from `BreakpointManager`, not from temp-BP read.
