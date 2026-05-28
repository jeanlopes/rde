<!--
SYNC IMPACT REPORT
==================
Version change: 3.0.0 → 4.0.0 (MAJOR)
Rationale for MAJOR bump: Removal of LLDB as a permitted debugger backend and elimination of the
dual-backend architecture. The project pivots from a generic debugger platform (RDC App) to a
Windows-native debugger engine built from scratch (RDE). LLDB, Python/PyO3, MCP, egui,
autonomous-agent, replay-engine, and multi-backend concepts are removed or deferred.

Removed principles:
  - I. Runtime Intelligence, Not Tool Wrapping (superseded by narrower Engine-First scope)
  - III. MCP as the AI-Debugger Contract (MCP layer removed from core; may be reintroduced later)
  - IV. Deterministic Replay (deferred to post-MVP)
  - V. Autonomous Agent Discipline (deferred to post-MVP)

Renamed/modified principles:
  - I. Engine-First, Not Wrapper-First (replaces Runtime Intelligence with focus on native engine)
  - III. REPL-First Architecture (new, replaces MCP contract)
  - IV. Windows-Native Purity (new, replaces dual-backend with Win32-only core)
  - VI. Rust Safety First: LLDB is now PROHIBITED (was explicitly permitted in v3.0.0)

Added sections:
  - Technology Stack & Architecture Constraints: workspace layout updated to RDE crates
  - Development Workflow: Constitution Check gate retained

Templates requiring updates:
  - .specify/templates/plan-template.md              ✅ no LLDB/agent refs; compatible
  - .specify/templates/spec-template.md              ✅ generic; compatible
  - .specify/templates/tasks-template.md             ✅ generic; compatible
  - README.md                                        ✅ created at project root
  - docs/quickstart.md                               ✅ created at docs/quickstart.md

Follow-up TODOs:
  - TODO(README): Create project README for Rust Debugger Engine (RDE)
  - TODO(docs): Write docs/quickstart.md for Windows setup and first debug session
  - TODO(MAINTAINERS): Define initial maintainer list for governance amendment approval
-->

# RDE Constitution

## Core Principles

### I. Engine-First, Not Wrapper-First (NON-NEGOTIABLE)

RDE MUST be a native debugger engine built from scratch on the Windows Debug API, not a wrapper
around LLDB, GDB, codelldb, or any external debugger process. Every debugging primitive — launch,
attach, breakpoint, single-step, memory read/write, register inspection, stack unwinding — MUST be
implemented directly via Win32 syscalls (`WaitForDebugEventEx`, `ReadProcessMemory`,
`WriteProcessMemory`, `GetThreadContext`, etc.).

Reference debuggers in C/C++ (TitanEngine, x64dbg) MAY be studied as living documentation of the
Win32 debugging model, but MUST NOT be linked or copied as dependencies.

**Rationale**: Wrapper architectures inherit the limitations of the wrapped tool. LLDB’s
blocking execution model, fragile FFI surface, and second-class Windows support are exactly the
problems RDE exists to solve. A native engine is the only path to runtime breakpoint injection,
async REPL responsiveness, and full architectural control.

### II. Crate-First Modularity (NON-NEGOTIABLE)

Every discrete capability MUST be a standalone Rust crate in the `crates/` workspace. Each crate
MUST:
- Have a single, clearly stated responsibility
- Be independently compilable and testable via `cargo test --package <crate>`
- Expose a public API with documented guarantees
- Carry no circular dependencies on other workspace crates

Binaries in `apps/` or at the workspace root MUST depend on `crates/`, never the reverse. No
capability may be siloed inside a binary without a corresponding library crate.

**Rationale**: Enforces independent testability, prevents monolithic binaries, and enables future
embedding of individual crates (e.g., `rde-core` in an IDE plugin) without pulling in the entire
platform.

### III. REPL-First Architecture (NON-NEGOTIABLE)

All debugger control MUST transit an async message-passing boundary between the UI/REPL thread and
the debug event loop. The REPL thread MUST remain responsive at all times — including while the
target process is running. Commands (`break`, `continue`, `step`, `read_mem`, etc.) are sent via
channels (`tokio::sync::mpsc` or `crossbeam-channel`); events (`BreakpointHit`, `ProcessExited`,
etc.) are received via a separate channel.

The debug loop thread (the one calling `WaitForDebugEventEx`) MUST perform no blocking I/O outside
of the Win32 debug APIs. Command dispatch MUST be non-blocking.

**Rationale**: LLDB’s inability to inject breakpoints during target execution is an architectural
failure caused by blocking, single-threaded design. REPL-first message passing is the antidote:
it enables runtime mutation of debugger state without freezing the user interface.

### IV. Windows-Native Purity (NON-NEGOTIABLE)

The core debugger engine MUST target Windows 10/11 (x86-64) exclusively. Linux or macOS support
MUST NOT be introduced into `crates/rde-win32` or `crates/rde-core`. Multi-platform abstractions
(e.g., a generic `ptrace` trait) are FORBIDDEN in the foundational crates.

Symbol resolution MUST use the Windows-native `DbgHelp.dll` API (`SymInitialize`, `SymFromAddr`,
`StackWalk64`). PDB is the only symbol format supported in the MVP. DWARF support MAY be discussed
only after the engine is feature-complete on Windows.

**Rationale**: Windows is the primary target and the reason LLDB is inadequate. Adding
multi-platform indirection now would dilute effort, introduce abstraction overhead, and reproduce
the exact "jack of all trades" failure mode that makes LLDB poor on Windows.

### V. Reference-Driven Development

TitanEngine and x64dbg MUST be treated as reference implementations — architectural textbooks, not
source material. Engineers MAY read their source to understand:
- How `WaitForDebugEvent` interacts with `ContinueDebugEvent`
- The INT3 (0xCC) breakpoint restore-step-reinstall pattern
- How to suspend threads before patching memory

Engineers MUST NOT copy C++ code, struct layouts, or naming conventions blindly into Rust.
Unsafe code MUST carry a safety proof. The Rust reimplementation MUST be idiomatic: ownership,
channels, enums, and trait-based backends where appropriate.

**Rationale**: C++ debuggers are excellent references for the Win32 debugging model, but their
memory models, error handling, and concurrency patterns do not translate directly to Rust. RDE
must be a Rust-native program, not a transpilation.

### VI. Rust Safety First — No External Runtimes

All production code MUST be written in Rust (stable toolchain). The following are FORBIDDEN:
- LLDB, liblldb.dll, lldb-sys, lldb-safe, or any LLVM debugger binding
- Python, PyO3, or any Python interpreter bridge
- External debugger processes (codelldb.exe, lldb.exe, gdb.exe)
- GDB as a backend or reference dependency

`unsafe` blocks are FORBIDDEN unless all three conditions are met:
1. An inline comment provides a complete safety proof at the `unsafe` block
2. A GitHub issue tracks the unsafe usage with a link in the comment
3. A safe alternative was considered and rejected with a written rationale in the same comment

`unsafe` is expected and accepted ONLY in `crates/rde-win32` for direct Windows API calls
(`ReadProcessMemory`, `WriteProcessMemory`, `CreateProcessW`, `WaitForDebugEventEx`, etc.).

Async code MUST use Tokio as the sole async runtime. No competing runtimes (async-std, smol) may
be introduced. The MSRV MUST be declared in `workspace.package` in the root `Cargo.toml`.

**Rationale**: LLDB and Python create invisible deployment failures, antivirus surface area, and
massive FFI fragility. A pure-Rust engine with documented `unsafe` perimeters is auditable,
testable, and deployable with `cargo build` alone.

### VII. Hackable by Design

The codebase MUST remain comprehensible to a single engineer in a single afternoon. Enterprise
abstraction layers, plugin systems, scripting engines, and remote-debugging protocols are
FORBIDDEN in the MVP. The engine MUST fit in the head of a maintainer.

Every public crate API (`pub` items at each crate root) MUST carry `///` doc comments and include
at least one usage example. Complex debugger behaviors (breakpoint shadowing, thread suspension
coordination, symbol caching) MUST be documented with inline diagrams or state-machine comments.

**Rationale**: A debugger is a systems tool that users will modify when it misbehaves. If the
code requires a committee to understand, it has failed its user. The goal is a small, fast,
modern engine — not a second LLDB.

## Technology Stack & Architecture Constraints

- **Primary language**: Rust (stable toolchain; MSRV declared in workspace `Cargo.toml`)
- **Target platform**: Windows 10/11 (x86-64). No Linux or macOS support is planned or claimed.
- **Debugger core**: Win32 Debug API via `windows-rs` (`WaitForDebugEventEx`, `CreateProcessW`,
  `DebugActiveProcess`, `ReadProcessMemory`, `WriteProcessMemory`, `GetThreadContext`,
  `SetThreadContext`, `SuspendThread`, `ResumeThread`)
- **Symbol engine**: `DbgHelp.dll` via `libloading` or equivalent (`SymInitialize`, `SymFromAddr`,
  `StackWalk64`, `SymGetLineFromAddr64`)
- **Demangling**: `rustc_demangle` for Rust symbol names
- **Disassembly**: Capstone Engine via `capstone` crate (x86-64 mode)
- **Async runtime**: Tokio only — no mixing with async-std or smol
- **REPL/CLI**: Async message passing (`tokio::sync::mpsc` or `crossbeam-channel`)
- **TUI (post-MVP)**: `ratatui`
- **Serialization**: `serde` with `serde_json` or `postcard` for config and snapshots
- **Logging**: `tracing` — all event-handling code paths MUST emit structured logs
- **Testing**: `cargo test`, `insta` for snapshots, `mockall` for mock backends
- **External dependencies policy**: ZERO runtime dependencies outside of Rust crates.
  `cargo build` on a fresh Windows machine with Rust installed MUST produce a working binary.
- **Forbidden**: LLDB, GDB, Python, PyO3, codelldb, liblldb.dll, DWARF (MVP), egui,
  remote-debugging protocols, JIT expression evaluation, plugin systems
- **Workspace layout** (authoritative):
  ```
  crates/
    rde-core/           # Traits, types, debug engine state machine, event channels
    rde-win32/          # Windows Debug API backend (windows-rs, unsafe Win32 calls)
    rde-symbols/        # PDB/DbgHelp integration, demangling, stack walking
    rde-breakpoint/     # Breakpoint manager: set/remove/hit/restore/reinstall
    rde-repl/           # Command parser, executor, REPL loop
    rde-cli/            # Binary entry point, argument parsing, session orchestration

  examples/
    hello_debuggee.rs   # Minimal target program for integration tests

  docs/
    win32-debug-api.md  # API reference map for maintainers
  ```

## Development Workflow

- **Spec-Driven**: Every feature MUST have a spec (`spec.md`) and implementation plan (`plan.md`)
  before any code is written. The mandated workflow is:
  `/speckit-specify` → `/speckit-clarify` → `/speckit-plan` → `/speckit-tasks` →
  `/speckit-implement`
- **Test-First**: For any new crate capability, tests MUST be written and confirmed failing before
  implementation begins. The Red-Green-Refactor cycle is strictly enforced.
- **Crate PR Scope**: Pull requests MUST NOT span more than two workspace crates unless the change
  is an atomic refactor updating all callers. Cross-crate feature work MUST be broken into
  sequential crate-scoped PRs reviewed independently.
- **Constitution Check in Plans**: Every `plan.md` MUST include a Constitution Check section
  verifying compliance with all seven Core Principles before Phase 0 research proceeds.
  Non-compliant plans MUST NOT advance to task generation.
- **Observability gate**: Any PR adding a new runtime-event-handling code path that lacks
  `tracing` instrumentation MUST be rejected at review.
- **Zero-install gate**: Any PR that introduces a dependency on a system-installed tool, runtime,
  or interpreter (LLDB, Python, etc.) MUST be rejected. All dependencies MUST be expressible in
  `Cargo.toml`.

## Governance

This constitution supersedes all other project practices, style guides, and informal conventions.
Amendments MUST follow this procedure:
1. Open a GitHub issue proposing the amendment, referencing the affected principle(s)
2. Obtain approval from at least one project maintainer (documented in the issue)
3. Include a migration plan for any in-flight specs or implementations affected
4. Increment `CONSTITUTION_VERSION` per the versioning policy below
5. Update `Last Amended` to the date of the merge commit

**Versioning policy**:
- MAJOR: Removal or backward-incompatible redefinition of a Core Principle or Technology Stack
  constraint (e.g., re-allowing LLDB, adding Linux support to core crates)
- MINOR: New principle or section added, or materially expanded guidance
- PATCH: Clarifications, wording fixes, non-semantic refinements

**Compliance review**: All pull requests MUST include a self-assessment of compliance with the
Core Principles in the PR description. Reviewers MUST reject PRs that violate principles without
an approved exception documented in this Governance section.

**Authoritative guidance**: This file (`.specify/memory/constitution.md`) is the single source of
truth for project governance. In any conflict between this document and other guidance files, this
constitution prevails.

**Version**: 4.0.0 | **Ratified**: 2026-05-20 | **Last Amended**: 2026-05-28
