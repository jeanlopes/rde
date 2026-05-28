# Data Model: Rust Debug Extensions

**Feature**: Rust Debug Extensions
**Date**: 2026-05-28

---

## Entity: `PrettyPrinter`

Represents a formatter for a specific Rust type.

| Field | Type | Description |
|-------|------|-------------|
| `type_name` | `String` | Fully-qualified Rust type name (e.g., `std::vec::Vec<i32>`) |
| `format_fn` | `fn(&MemoryReader, Address, &FormatBudget) -> PrettyValue` | Formatter implementation |
| `priority` | `u32` | Matching priority (higher = more specific; e.g., generic `Vec<T>` vs specialized `Vec<u8>`) |

**Relationships**:
- Owned by `PrinterRegistry` (one registry per engine session).
- Consumes `MemoryReader` from `rde-win32` to fetch bytes.
- Consumes type info from `rde-symbols` to resolve type names.

---

## Entity: `FormatBudget`

Tracks resource limits during recursive pretty printing.

| Field | Type | Description |
|-------|------|-------------|
| `max_depth` | `u32` | Maximum recursion depth remaining |
| `max_elements` | `u32` | Maximum collection elements remaining |
| `max_bytes` | `usize` | Maximum total bytes to read from target memory |

**Validation rules**:
- `max_depth` defaults to 5; must not underflow (saturates at 0).
- `max_elements` defaults to 100; decrements per collection item.
- `max_bytes` defaults to 4096; decrements by bytes read per memory fetch.
- When any field reaches zero, output `"..."` (truncated) and stop descending.

---

## Entity: `PrettyValue`

The result of pretty-printing a value.

| Variant | Payload | Description |
|---------|---------|-------------|
| `Scalar` | `String` | Primitive or simple value (e.g., `42`, `"hello"`) |
| `Enum` | `String, Option<Box<PrettyValue>>` | Enum variant name + optional payload (e.g., `Some(Box<PrettyValue>)`) |
| `Sequence` | `Vec<PrettyValue>` | Ordered collection (e.g., `[10, 20, 30]`) |
| `Map` | `Vec<(PrettyValue, PrettyValue)>` | Key-value pairs (e.g., `{1: "a", 2: "b"}`) |
| `Raw` | `String` | Fallback: raw memory representation when no printer matches |
| `Truncated` | — | Indicator that budget was exhausted |

**State transitions**:
- `MemoryBytes` → `PrettyValue` (via `PrettyPrinter::format_fn`)
- `PrettyValue` → `String` (via `Display` implementation for REPL output)

---

## Entity: `AsyncTask`

Represents a Tokio task in the debugged process.

| Field | Type | Description |
|-------|------|-------------|
| `task_id` | `u64` | Unique task identifier (from Tokio runtime) |
| `state` | `TaskState` | Current execution state |
| `function_name` | `Option<String>` | Name of the async function or spawn point (from PDB symbols) |
| `runtime_thread_id` | `Option<u32>` | OS thread ID of the Tokio worker thread currently running the task |

**Validation rules**:
- `task_id` must be non-zero.
- `state` must be one of the defined enum variants.
- `function_name` may be `None` if symbols are stripped or the task is a closure.

---

## Entity: `TaskState`

Enum of possible Tokio task states.

| Variant | Description |
|---------|-------------|
| `Running` | Task is actively executing on a worker thread |
| `Idle` | Task is scheduled but not currently running |
| `Sleeping` | Task is waiting on an I/O or timer (parked waker) |
| `Completed` | Task has finished and its JoinHandle is ready |

---

## Entity: `CargoProject`

Represents a Cargo workspace/project context for launch.

| Field | Type | Description |
|-------|------|-------------|
| `manifest_path` | `PathBuf` | Absolute path to `Cargo.toml` |
| `workspace_root` | `PathBuf` | Absolute path to workspace root (from `cargo metadata`) |
| `default_package` | `String` | Default package name (for non-workspace projects) |

---

## Entity: `CargoTarget`

A specific build target within a Cargo project.

| Field | Type | Description |
|-------|------|-------------|
| `package_name` | `String` | Package containing the target |
| `target_name` | `String` | Name of the bin/lib/test target |
| `target_kind` | `CargoTargetKind` | `Bin`, `Lib`, `Test`, `Bench`, `Example` |
| `profile` | `String` | Build profile (`dev` / `release` or custom) |
| `features` | `Vec<String>` | Active Cargo features |
| `artifact_path` | `PathBuf` | Resolved path to the compiled executable |

**Validation rules**:
- `artifact_path` must point to an existing file when the build is fresh.
- `profile` must be a valid profile declared in `Cargo.toml` or a built-in profile.
- `features` must be a subset of the package's declared features.

---

## Entity: `CargoTargetKind`

| Variant | Description |
|---------|-------------|
| `Bin` | Binary executable |
| `Lib` | Library crate |
| `Test` | Test target |
| `Bench` | Benchmark target |
| `Example` | Example target |

---

## Relationships Diagram

```
+---------------+       uses       +------------------+
|  rde-repl     |<---------------->| rde-pretty-print |
+---------------+                  +------------------+
       |                                  |
       | uses                        reads memory
       v                                  v
+------------------+              +------------------+
|   rde-tokio      |              |    rde-win32     |
+------------------+              +------------------+
       |                                  |
       | uses                             | uses
       v                                  v
+------------------+              +------------------+
|   rde-core       |<------------>|   rde-symbols    |
| (channels,       |   uses       | (PDB, demangle)  |
|  engine state)   |              |                  |
+------------------+              +------------------+
       ^
       | uses
       v
+------------------+
|   rde-cargo      |
| (metadata,       |
|  build trigger)  |
+------------------+
       |
       | uses
       v
+------------------+
|   rde-cli        |
| (entry point)    |
+------------------+
```
