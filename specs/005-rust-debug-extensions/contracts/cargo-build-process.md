# Contract: Cargo Build Process Management

> Scope: `rde-cargo` crate — spawning and monitoring `cargo build` as a pre-launch step.

---

## Contract: `CreateProcessW` for `cargo build`

### Pre-conditions
- `cargo.exe` is resolvable from `PATH` or `CARGO_HOME`.
- Working directory contains a valid `Cargo.toml`.
- Command-line arguments are validated (package name exists, target exists, features are declared).

### Post-conditions (Ok)
- Child process `cargo build` is created with `CREATE_NEW_PROCESS_GROUP`.
- Stdout and stderr are captured to pipes for potential display.
- Process handle is valid for `WaitForSingleObject`.

### Post-conditions (Err)
- `ERROR_FILE_NOT_FOUND`: `cargo.exe` not in PATH.
- `ERROR_INVALID_PARAMETER`: Malformed command line.
- No child process is created; caller must report error to REPL.

### Invariant
- The debuggee MUST NOT be launched until `cargo build` exits with code 0.
- If `cargo build` exits non-zero, the launch MUST be aborted and stderr output forwarded to the user.

---

## Contract: `WaitForSingleObject` (build completion)

### Pre-conditions
- Valid process handle from successful `CreateProcessW`.
- Timeout is a `DWORD` in milliseconds (recommended: 300,000 ms = 5 minutes).

### Post-conditions (Ok)
- `WAIT_OBJECT_0`: Process exited. Exit code must be retrieved via `GetExitCodeProcess`.
- Exit code 0: build succeeded; artifact path is valid.
- Exit code non-zero: build failed; do not launch debuggee.

### Post-conditions (Err)
- `WAIT_TIMEOUT`: Build is still running. Caller may:
  - Show a progress indicator in REPL and yield (async await).
  - Retry wait with same timeout.
- `WAIT_FAILED`: System error. Do not launch debuggee.

### Invariant
- While waiting, the REPL thread MUST remain responsive (async yield between wait retries).
- The debug loop thread MUST NOT be involved in waiting for `cargo build`.

---

## Contract: Artifact Staleness Check

### Pre-conditions
- `CargoTarget.artifact_path` is resolved.
- Source directory (`src/`) exists.

### Post-conditions (Ok)
- If artifact mtime ≥ newest source mtime: artifact is fresh; skip build.
- If artifact missing or mtime < newest source mtime: artifact is stale; trigger build.

### Invariant
- Staleness check is a heuristic. It may produce false positives (rebuild when not strictly needed) but must not produce false negatives (skip build when source changed).
- If `cargo build` is triggered, the previous artifact is overwritten; no cleanup required.

---

## Inline Safety Proof

```rust
// INVARIANT: cargo build and debuggee launch are sequential, never concurrent.
//            The debuggee process handle is created only after cargo build exits 0.
//            This prevents attaching to a binary that is mid-write.
//
// VIOLATION: If the debuggee is launched before WaitForSingleObject returns,
//            the executable on disk may be incomplete, causing CreateProcessW
//            to fail or launch a corrupted image.
//            Tracked by: rde#cargo-build-race (placeholder issue link)
```
