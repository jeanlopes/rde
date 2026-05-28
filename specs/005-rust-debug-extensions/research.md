# Research: Rust Debug Extensions

**Feature**: Rust Debug Extensions (pretty printers, Tokio tasks, Cargo integration)
**Date**: 2026-05-28

---

## Unknown 1: Rust std type memory layouts for pretty printing

**Research Task**: Determine stable memory layouts for `Option<T>`, `Vec<T>`, `String`, `HashMap<K,V>`, `BTreeMap<K,V>`, `Result<T,E>` on Windows x86-64.

**Decision**: Use documented ABI-stable layouts verified against PDB type information.

**Rationale**:
- `Option<T>`: Discriminant is a byte prefix (`0` for `None`, `1` for `Some` when `T` is not a niche type; niche optimization may flip this). On x86-64 with PDB, `SymGetTypeInfo` provides discriminant values and variant layouts.
- `Vec<T>`: Triple `(ptr, len, cap)` — all `usize`. Order is guaranteed stable since Rust 1.0.
- `String`: Wrapper around `Vec<u8>` — same triple layout, UTF-8 bytes at `ptr`.
- `HashMap<K,V>`: Uses `hashbrown` raw table. Layout is `(base: NonNull<Control>, ctrl: NonNull<Control>, bucket_mask: usize, items: usize, growth_left: usize)`. This is less stable; MVP will print a summary (`HashMap { len: N, capacity: M }`) with optional deep iteration up to a limit.
- `BTreeMap<K,V>`: Root node pointer + length. Deep inspection is complex; MVP will print summary only.
- `Result<T,E>`: Same enum layout as `Option` — discriminant + payload.

**Alternatives considered**:
- Use `gimli`/DWARF type info — REJECTED. Constitution mandates PDB-only for MVP.
- Hard-code layouts without PDB verification — REJECTED. Violates Contract-First principle. PDB type info must be the source of truth for offsets and sizes.

---

## Unknown 2: Tokio runtime memory structures for task inspection

**Research Task**: Determine how to locate and read Tokio task states from a debugged process's memory.

**Decision**: Best-effort heuristic scanning for Tokio 1.x runtime structures, with graceful degradation when runtime is not detected.

**Rationale**:
- Tokio's runtime internals are not a stable public API. However, the runtime global is typically reachable via a static or thread-local pointer.
- Approach: scan module memory for known Tokio runtime vtable / signature patterns, then follow documented field offsets for `Scheduler` and `Task` structures.
- Task state can be inferred from the task's header bits (`RUNNING`, `NOTIFIED`, `CANCELLED`).
- Function name is resolved via PDB symbols on the task's future vtable or poll function pointer.

**Alternatives considered**:
- Require debuggee to link a special helper crate — REJECTED. Violates zero-install principle; user should not modify their code.
- Parse DWARF unwind info to find async frames — REJECTED. PDB-only; stack walking via `StackWalk64` can supplement but is not the primary mechanism.

**Limitation documented**: Tokio task inspection is version-dependent. If structure offsets change, the feature degrades to "runtime detected but tasks unreadable" rather than crashing.

---

## Unknown 3: Cargo metadata and target resolution

**Research Task**: Determine the best way to resolve the binary path and build configuration from a Cargo project.

**Decision**: Use `cargo metadata --format-version 1` (machine-readable JSON) to resolve package targets, then derive the expected artifact path in `target/<profile>/`.

**Rationale**:
- `cargo metadata` is the official, stable interface for querying Cargo project structure. It returns packages, targets, dependencies, and workspace membership.
- For a given package + target + profile + features, the artifact path follows a deterministic pattern: `<workspace-root>/target/<profile>/<target-name>.exe` (for `bin` targets on Windows).
- Staleness check: compare mtime of the artifact against the newest mtime of all `.rs` files in the package's `src/` directory (simple heuristic; matches Cargo's own behavior).
- Build trigger: invoke `cargo build --package <pkg> --bin <bin> --profile <profile> --features <features>` and wait for completion, capturing exit code and stdout/stderr.

**Alternatives considered**:
- Parse `Cargo.toml` manually with `toml` crate — REJECTED. Fragile: does not handle workspaces, target defaults, or feature resolution correctly.
- Hard-code `target/debug/` paths — REJECTED. Fails for workspaces, custom profiles, and renamed targets.

---

## Unknown 4: Pretty printer architecture and recursion limits

**Research Task**: Design the internal architecture for pretty printers to handle nested types safely.

**Decision**: Registry pattern with typed dispatch and a configurable recursion/length budget.

**Rationale**:
- Registry: a map from fully-qualified type name (from PDB) to a `PrettyPrinter` trait object. `rde-pretty-print` registers built-in printers at startup.
- Typed dispatch: `rde-symbols` resolves the type name via PDB; `rde-pretty-print` looks up the printer and invokes it with a `MemoryReader` handle and a `FormatBudget`.
- `FormatBudget` tracks: remaining depth (default 5), remaining elements (default 100), total bytes to read (default 4 KiB). When budget is exhausted, output `"..."` (truncated).
- Circular reference safety: budget prevents infinite recursion; no need for reference-cycle detection in MVP.

**Alternatives considered**:
- Visitor pattern with callbacks — REJECTED. More complex than needed for MVP; registry+dispatch is simpler and testable.
- No budget, rely on type system to prevent cycles — REJECTED. Rust allows `Rc<RefCell<Self>>` and other cyclic types; debugger must not hang.

---

## Unknown 5: REPL integration points

**Research Task**: Determine how pretty printers and task commands integrate with the existing REPL command parser and async channel protocol.

**Decision**: Extend the REPL command enum and the engine request/response channel protocol with new variants.

**Rationale**:
- New REPL commands: `print <expr>`, `vars`, `tasks`, `cargo debug [opts]`.
- Engine request enum (`EngineRequest`) gets variants: `Print { frame_id, expression }`, `ListTasks`, `CargoLaunch { manifest_path, package, target, profile, features }`.
- Engine response enum (`EngineResponse`) gets variants: `PrettyValue(PrettyValue)`, `TaskList(Vec<AsyncTask>)`, `CargoLaunchResult(Result<ProcessId, CargoError>)`.
- These cross the async channel between REPL thread and debug loop thread, keeping the REPL responsive.

**Alternatives considered**:
- Synchronous blocking calls from REPL — REJECTED. Violates REPL-First Architecture (Principle III).
- Direct function calls bypassing channels — REJECTED. Breaks the architectural boundary and makes testing harder.
