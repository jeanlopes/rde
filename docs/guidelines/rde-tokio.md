# rde-tokio

Inspection of Tokio async task runtime in the debuggee.

---

## Overview

`rde-tokio` scans a debuggee process for a Tokio runtime and extracts information about active async tasks: their IDs, states, and associated function names.

## Limitations

- **Version-dependent**: Tokio's internal structures are not a stable API. The scanner uses heuristics that work with Tokio 1.x but may break with future versions.
- **Best-effort**: If the runtime structure changes, the scanner degrades gracefully with an informative message rather than crashing.

## API

### TokioScanner

```rust
use rde_tokio::scanner::TokioScanner;
use rde_tokio::task::AsyncTask;
use rde_core::DebugError;

let scanner = TokioScanner::new();
let tasks = scanner.list_tasks(process_id)?;
```

### AsyncTask

```rust
pub struct AsyncTask {
    pub task_id: u64,
    pub state: TaskState,
    pub function_name: Option<String>,
    pub runtime_thread_id: Option<u32>,
}
```

### TaskState

```rust
pub enum TaskState {
    Running,   // Actively executing on a worker thread
    Idle,      // Scheduled but not currently running
    Sleeping,  // Waiting on I/O or timer
    Completed, // Finished execution
}
```

## Usage

### From the REPL

```bash
rde> tasks
ID    State      Function                Thread
----  ---------  ----------------------  ------
1     Running    process_requests        12345
2     Sleeping   heartbeat_timer         12346
3     Idle       background_worker       12347
```

### Programmatic API

```rust
use rde_tokio::scanner::TokioScanner;

let scanner = TokioScanner::new();
match scanner.list_tasks(pid) {
    Ok(tasks) => {
        for task in tasks {
            println!(
                "Task {}: {:?} - {:?} (thread: {:?})",
                task.task_id,
                task.state,
                task.function_name,
                task.runtime_thread_id
            );
        }
    }
    Err(e) => eprintln!("Failed to list tasks: {}", e),
}
```

## Implementation Details

### Runtime Detection (MVP)

The current MVP implementation:
1. Scans module memory for known Tokio runtime signatures
2. Follows scheduler internal links to enumerate tasks
3. Decodes task state from header bits

### Function Name Resolution

When available via PDB symbols:
- Uses `rde-symbols` to resolve the task's poll function pointer or vtable
- Falls back to "unknown" if symbols are stripped

### Graceful Degradation

If no Tokio runtime is detected:

```rust
vec![AsyncTask {
    task_id: 0,
    state: TaskState::Completed,
    function_name: Some("No Tokio runtime detected".to_string()),
    runtime_thread_id: None,
}]
```

## Testing

### Unit Tests

```rust
#[test]
fn test_tokio_scanner_no_runtime() {
    let scanner = TokioScanner::new();
    let tasks = scanner.list_tasks(1234).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_id, 0);
    assert_eq!(tasks[0].state, TaskState::Completed);
    assert_eq!(tasks[0].function_name, Some("No Tokio runtime detected".to_string()));
}
```

## Performance

Benchmark in `crates/rde-tokio/tests/perf_bench.rs`.

| Benchmark | Target | Status |
|-----------|--------|--------|
| Task listing (no runtime) | < 2s | ✅ Passing |

## Future Work

- [ ] Implement real Tokio 1.x runtime signature scanning
- [ ] Support multiple Tokio runtime versions
- [ ] Extract task waker information
- [ ] Show task spawn location (file:line)
