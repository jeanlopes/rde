# Implementation Plan: Thread and Module Tracking

**Branch**: `002-thread-module-tracking` | **Date**: 2026-05-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-thread-module-tracking/spec.md`

## Summary

Implement dynamic thread and module (DLL) tracking for the RDE Windows debugger. This feature adds
thread registries with selection, module registries with symbol-engine integration, and REPL
commands to list and switch threads. The work spans `rde-core` (engine state), `rde-win32` (debug
event extraction and handle caching), `rde-symbols` (DbgHelp module loading), and `rde-repl`
(command parsing and event display).

## Technical Context

**Language/Version**: Rust 1.78 (stable) — MSRV declared in workspace `Cargo.toml`

**Primary Dependencies**: `windows` 0.56 (Win32 Debug API), `libloading` 0.8 (DbgHelp.dll), `tokio`
1.37 (async runtime), `tracing` 0.1 (structured logging)

**Storage**: N/A — runtime in-memory registries (`HashMap`) inside the debug engine session

**Testing**: `cargo test`, `insta` for snapshot testing, `mockall` for mock backends

**Target Platform**: Windows 10/11 (x86-64) exclusively

**Project Type**: Systems CLI tool with library crates (debugger engine)

**Performance Goals**: Thread list < 200ms for 50+ threads; module list < 300ms; event processing
< 100ms per thread/module event

**Constraints**: No LLDB, no Python, no external debugger processes; `unsafe` only in
`crates/rde-win32`; all new event-handling code paths MUST emit `tracing` logs

**Scale/Scope**: Single local process debugging; thread counts up to OS limits (thousands);
module counts typical for Windows processes (10–200 DLLs)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Engine-First | ✅ PASS | All work uses Win32 Debug API directly; no wrappers |
| II. Crate-First | ✅ PASS | Changes split across `rde-core`, `rde-win32`, `rde-symbols`, `rde-repl` |
| III. REPL-First | ✅ PASS | New commands (`ListThreads`, `SelectThread`, `ListModules`) use existing channel architecture |
| IV. Windows-Native | ✅ PASS | No cross-platform abstractions introduced |
| V. Reference-Driven | ✅ PASS | Thread/module patterns align with TitanEngine/x64dbg reference behavior |
| VI. Rust Safety | ✅ PASS | `unsafe` confined to `rde-win32` for Win32 API calls; safety proofs required |
| VII. Hackable | ✅ PASS | No enterprise layers; thread registry is a `HashMap` |
| VIII. Contract-First | ✅ PASS | New Win32 APIs get contract docs in `docs/contracts/` and inline invariants |

## API Contracts & Invariants

*GATE: Required for any plan touching `crates/rde-win32` or the debug loop / engine state machine.*

| API / Boundary | Pre-conditions | Post-conditions (Ok) | Post-conditions (Err) | Invariant |
|----------------|---------------|----------------------|-----------------------|-----------|
| `OpenThread` | Valid `ThreadId` exists in target process | Handle has `THREAD_ALL_ACCESS` | `ERROR_INVALID_PARAMETER` if TID invalid | Must be paired with `CloseHandle` |
| `GetMappedFileNameW` | Valid process handle; valid memory address mapped to a file | Buffer contains NT path (e.g., `\Device\HarddiskVolume...`) | `ERROR_INVALID_PARAMETER` if address not mapped | Path may need `GetLogicalDriveStrings` translation |
| `SymLoadModuleEx` | `SymInitializeW` called for process; valid module base and size | Symbols loaded for module; addresses in module resolvable | `ERROR_NOT_SUPPORTED` if no matching PDB | Must be called before resolving addresses in new module |
| `EnumProcessModules` | Process handle with `PROCESS_QUERY_INFORMATION \| PROCESS_VM_READ` | Array of `HMODULE` handles populated | `ERROR_PARTIAL_COPY` if process 32-bit from 64-bit debugger | Use `K32GetModuleInformation` for size |
| `ReadProcessMemory` (for module name) | Valid process handle; `lpImageName` pointer from `LOAD_DLL_DEBUG_INFO` | Buffer contains UTF-16 path string | `ERROR_PARTIAL_COPY` if page not readable | Only valid during debug event processing |

**Type-system distinctions:**
- [x] Does this feature introduce system-vs-user event ambiguity? **No** — thread and module events are purely system notifications; no user-vs-system ambiguity.
- [x] Are magic values used? **Yes, `thread_id: 0` is currently used as "default" in `StepInto` and `ReadRegisters`. Plan refactors this to `Option<ThreadId>` with `selected_thread` fallback.**

**Golden path:**
- Expected sequence: `ProcessLaunched` → `ThreadCreated(main_tid)` → `ModuleLoaded(exe)` → `ModuleLoaded(ntdll)` → ... → `ThreadCreated(new_tid)` → `ThreadExited(new_tid)` → `ModuleLoaded(new_dll)` → `ModuleUnloaded(new_dll)` → `ProcessExited`
- Snapshot stored at: `test_data/golden_paths/002-thread-module-tracking.txt`

## Project Structure

### Documentation (this feature)

```text
specs/002-thread-module-tracking/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit-tasks command)
```

### Source Code (repository root)

```text
crates/
  rde-core/
    src/
      lib.rs          # Thread, Module types already defined; Session expands
      engine.rs       # Add thread/module registries, selected_thread, event handlers
      events.rs       # Add SelectThread command, ModuleUnloaded event
  rde-win32/
    src/
      debug_loop.rs   # Extract hThread, lpBaseOfDll, hFile from debug events
      thread.rs       # Add cached-handle variants; keep raw variants for safety
      module.rs       # Implement enumerate_modules via EnumProcessModules
      lib.rs          # Export new module utilities
  rde-symbols/
    src/
      dbghelp.rs      # Wire load_module to SymLoadModuleEx
      lib.rs          # SymbolEngine trait may need load_module signature refinement
  rde-repl/
    src/
      parser.rs       # Add `thread <id>` command
      lib.rs          # Display ThreadCreated, ThreadExited, ModuleLoaded, ModuleUnloaded
  rde-cli/
    src/
      main.rs         # No changes expected (orchestration layer)

tests/
  integration_tests.rs  # Add thread/module tracking integration tests

test_data/
  golden_paths/
    002-thread-module-tracking.txt
```

**Structure Decision**: Workspace crate layout already established by spec 001. This feature adds
capabilities within existing crates rather than introducing new crates, consistent with
Constitution Principle II (Crate-First Modularity) because thread/module tracking is not a
discrete standalone capability — it is core engine state management.

## Complexity Tracking

> No Constitution Check violations require justification.

## Phase 0: Research Decisions

See [research.md](research.md) for full details. Key decisions:

1. **Thread handle caching**: Cache `hThread` from `CREATE_THREAD_DEBUG_EVENT` in the `Thread`
   struct instead of opening/closing per API call. This satisfies FR-012 and SC-006.
2. **Module name resolution**: Use `GetMappedFileNameW` as primary; fallback to
   `ReadProcessMemory` reading the `lpImageName` pointer from `LOAD_DLL_DEBUG_INFO` if the
   mapped file name is unavailable.
3. **Symbol engine integration**: Call `SymLoadModuleEx` (or rely on DbgHelp auto-load via
   search path) when `ModuleLoaded` is processed. The current `load_module` stub becomes a real
   implementation.
4. **Selected thread default**: The initial thread from `CREATE_PROCESS_DEBUG_EVENT` becomes
   the default selected thread. If it exits, the oldest remaining running thread becomes selected.
