# Implementation Plan: Rust Debug Extensions

**Branch**: `005-rust-debug-extensions` | **Date**: 2026-05-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/005-rust-debug-extensions/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implement three Rust-specific debugging capabilities atop the native Win32 engine: (1) pretty printers for standard Rust types (`Option`, `Vec`, `String`, `HashMap`, `Result`) that render readable values instead of raw memory layouts; (2) async task inspection for Tokio runtimes, showing task states and spawn origins; (3) Cargo project integration that resolves the correct debug target, triggers `cargo build` when stale, and launches the debuggee automatically. All features are accessed through the REPL and communicate via the existing async channel boundary.

## Technical Context

**Language/Version**: Rust (stable toolchain; MSRV declared in workspace `Cargo.toml`)

**Primary Dependencies**: `windows-rs`, `libloading` (DbgHelp), `capstone`, `tokio`, `serde`, `tracing`, `rustc_demangle`, `insta`, `mockall`

**Storage**: N/A — all data is read from the target process's memory or derived from Cargo metadata

**Testing**: `cargo test`, `insta` for snapshot testing of pretty-printer output, `mockall` for mocking memory-read backends

**Target Platform**: Windows 10/11 x86-64

**Project Type**: CLI debugger tool + library crates

**Performance Goals**: Pretty-print `Vec<T>` with ≤100 elements in <1s; list all Tokio tasks in <2s; Cargo launch ≤5s including build check

**Constraints**: Zero runtime dependencies outside Rust crates; no LLDB/Python/PyO3; REPL-first async message passing; `unsafe` only in `rde-win32` with three-condition safety proof

**Scale/Scope**: MVP covers `std`/`core` types (`Option`, `Vec`, `String`, `HashMap`, `BTreeMap`, `Result`) + Tokio 1.x task runtime inspection + Cargo target resolution and stale-build trigger

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Engine-First | ✅ PASS | Pretty printers, Tokio tasks, and Cargo integration are pure layers atop the native Win32 engine. No external debugger wrapping. |
| II. Crate-First | ✅ PASS | Three new standalone crates proposed (`rde-pretty-print`, `rde-tokio`, `rde-cargo`), each with a single responsibility and no circular deps. |
| III. REPL-First | ✅ PASS | All features are accessed via REPL commands (`print`, `vars`, `tasks`, `cargo debug`) and communicate through existing async channels (`tokio::sync::mpsc`). Debug loop remains non-blocking. |
| IV. Windows-Native Purity | ✅ PASS | Cargo resolution is pre-launch orchestration; debuggee execution stays Win32-only. No cross-platform abstractions introduced into `rde-core` or `rde-win32`. |
| V. Reference-Driven | ✅ PASS | No C++ code copied. Memory layouts are reverse-engineered from Rust ABI documentation and validated against PDB type info. |
| VI. Rust Safety First | ✅ PASS | No LLDB, Python, PyO3, or external debugger processes. Memory reads use existing `rde-win32` safe wrappers or new wrappers with documented safety proofs. |
| VII. Hackable by Design | ✅ PASS | Three focused crates, each expected <2 kLOC. No plugin systems, scripting engines, or remote-debugging protocols. |
| VIII. Contract-First | ✅ PASS | `ReadProcessMemory` contract documented for pretty-print reads. New contract file added to `docs/contracts/` for `cargo build` process management. Type system distinguishes `RawBytes` from `PrettyValue` via enum. |

## API Contracts & Invariants

*GATE: Required for any plan touching `crates/rde-win32` or the debug loop / engine state machine.*

This feature reads target memory and may spawn child processes. The following contracts apply:

| API / Boundary | Pre-conditions | Post-conditions (Ok) | Post-conditions (Err) | Invariant |
|----------------|---------------|----------------------|-----------------------|-----------|
| `ReadProcessMemory` (pretty-print path) | Process handle has `PROCESS_VM_READ`; target process is stopped (breakpoint hit); address validated by PDB type info | Buffer contains bytes from target address | Buffer undefined; `GetLastError` set | Only called when debuggee is stopped; never on a running thread |
| `SymGetTypeInfo` / `SymFromAddr` | `SymInitializeW` completed for process; module with PDB loaded | Type index / symbol info retrieved | Type info unavailable; do not infer layout without it | Call only when process is stopped; cache type indices per module |
| `CreateProcessW` (`cargo build`) | `cargo.exe` in PATH; `Cargo.toml` valid; working directory is a Cargo project | Child `cargo build` process created with stdout/stderr pipes | No process created; readable error message returned to REPL | Must capture exit code; only launch debuggee on exit code 0 |
| `WaitForSingleObject` (build wait) | Valid process handle from `CreateProcessW` | Build completed; exit code available | Timeout or abnormal termination | Use a generous timeout (e.g., 5 min) with periodic async yield to keep REPL responsive |

**Type-system distinctions:**
- [x] Does this feature introduce system-vs-user event ambiguity? **No.** Pretty-print reads are data queries, not debug events. Tokio task scanning is a data query. Cargo build is pre-launch orchestration.
- [x] Are magic values used to distinguish categories? **No.** `ValueKind` enum distinguishes `Pretty` vs `Raw`. `TaskState` enum distinguishes `Running`, `Idle`, `Sleeping`, `Completed`.

**Golden path:**
- **Pretty printer**: `BreakpointHit` → REPL command `print v` → channel to debug loop → `ReadProcessMemory` + `SymGetTypeInfo` → `rde-pretty-print` formats → channel back → REPL displays `Some(42)`.
- **Tokio tasks**: `BreakpointHit` → REPL command `tasks` → channel to debug loop → `rde-tokio` scans memory for Tokio runtime structures → extracts task states → channel back → REPL displays task table.
- **Cargo integration**: CLI `cargo debug` → `rde-cargo` reads `Cargo.toml` / runs `cargo metadata` → resolves target binary path → checks mtime vs source → runs `cargo build` if stale → `rde-core` launches debuggee with resolved path.

- [x] After implementation, what is the expected event sequence for the primary success scenario? Documented above.
- [x] Where will the golden path snapshot be stored? `test_data/golden_paths/005-rust-debug-extensions.txt`

## Project Structure

### Documentation (this feature)

```text
specs/005-rust-debug-extensions/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
  rde-core/           # Traits, types, engine state machine, channels
  rde-win32/          # Windows API backend (unsafe zone)
  rde-symbols/        # DbgHelp integration, demangling, stack walking
  rde-breakpoint/     # Breakpoint manager
  rde-repl/           # Command parser, REPL loop
  rde-tui/            # TUI interface (ratatui)
  rde-cli/            # Binary entry point
  rde-pretty-print/   # NEW: Rust value pretty printers (Option, Vec, String, etc.)
  rde-tokio/          # NEW: Tokio async task inspection
  rde-cargo/          # NEW: Cargo project integration, target resolution, build trigger

tests/
  contract/
  integration/
  unit/
```

**Structure Decision**: Crate-First Modularity (Constitution Principle II). Each new capability is a standalone crate:
- `rde-pretty-print` — reads target memory via `rde-win32` and formats Rust std types.
- `rde-tokio` — scans target memory for Tokio runtime structures and reports task states.
- `rde-cargo` — parses Cargo metadata, resolves binary paths, and invokes `cargo build` when stale.

`rde-repl` depends on `rde-pretty-print` and `rde-tokio` to serve `print`/`vars`/`tasks` commands.
`rde-cli` depends on `rde-cargo` to serve `cargo debug` launch flow.

## Complexity Tracking

> **No Constitution violations identified.** All new crates are justified by Principle II (Crate-First Modularity) and no simpler alternative exists that preserves independent testability.
