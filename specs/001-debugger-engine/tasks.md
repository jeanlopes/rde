# Tasks: Rust Debugger Engine (RDE)

**Input**: Design documents from `specs/001-debugger-engine/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Test tasks included per constitution Test-First requirement.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and workspace structure

- [X] T001 Create workspace `Cargo.toml` at repository root with MSRV 1.78+ and workspace members
- [X] T002 [P] Create crate directories: `crates/rde-core`, `crates/rde-win32`, `crates/rde-symbols`, `crates/rde-breakpoint`, `crates/rde-repl`, `crates/rde-cli`
- [X] T003 [P] Configure base dependencies in each crate `Cargo.toml` (`tokio`, `tracing`, `serde`, `thiserror`)
- [X] T004 Create `examples/hello_debuggee.rs` — minimal target program with main function, loop, and exit
- [X] T005 Configure `windows` crate features in `crates/rde-win32/Cargo.toml` (`Win32_System_Diagnostics_Debug`, `Win32_System_Threading`, `Win32_System_Memory`, `Win32_Foundation`, `Win32_Security`)
- [X] T006 Add `libloading`, `capstone`, `rustc_demangle`, `mockall`, `insta` to appropriate crate `Cargo.toml` files

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T007 Define `DebugBackend` trait in `crates/rde-core/src/lib.rs` (launch, attach, continue, step, read/write memory, get/set registers, suspend/resume thread)
- [X] T008 Define base types in `crates/rde-core/src/lib.rs`: `SessionId`, `ThreadId`, `BreakpointId`, `ProcessHandle`, `RawHandle`
- [X] T009 Define `DebugSession`, `Target`, `SessionState`, `PauseReason` in `crates/rde-core/src/lib.rs`
- [X] T010 Define `EngineCommand` enum in `crates/rde-core/src/events.rs`
- [X] T011 Define `EngineEvent` enum in `crates/rde-core/src/events.rs`
- [X] T012 Define `DebugError` enum in `crates/rde-core/src/lib.rs`
- [X] T013 Implement channel wrappers in `crates/rde-core/src/channel.rs` (`tokio::sync::mpsc` unbounded channels)
- [X] T014 Create stub `WindowsBackend` struct in `crates/rde-win32/src/lib.rs` implementing `DebugBackend`
- [X] T015 Implement `CreateProcessW` with `DEBUG_ONLY_THIS_PROCESS` in `crates/rde-win32/src/process.rs`
- [X] T016 Implement `DebugActiveProcess` (attach) in `crates/rde-win32/src/process.rs`
- [X] T017 Implement basic debug loop with `WaitForDebugEventEx` in `crates/rde-win32/src/debug_loop.rs`
- [X] T018 Implement `ContinueDebugEvent` in `crates/rde-win32/src/debug_loop.rs`
- [X] T019 Map all `DEBUG_EVENT_CODE` variants to Rust enum in `crates/rde-win32/src/debug_loop.rs`
- [X] T020 Configure `tracing` subscriber and add `#[instrument]` spans to all event-handling code paths

**Checkpoint**: Foundation ready — `cargo build --workspace` compiles; debug loop can spawn and receive CREATE_PROCESS_DEBUG_EVENT.

---

## Phase 3: User Story 1 - Controle de Execução e Breakpoints (Priority: P1) 🎯 MVP

**Goal**: Launch, attach, continue, single-step, and set/remove breakpoints with runtime injection.

**Independent Test**: Launch `hello_debuggee.exe`, set breakpoint at `main`, continue, verify program stops at breakpoint, then step once and continue to exit.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T021 [P] Create mock `DebugBackend` using `mockall` in `crates/rde-core/tests/mock_backend.rs`
- [ ] T022 [P] Write unit test for `DebugEngine` state transitions in `crates/rde-core/tests/engine_tests.rs`
- [ ] T023 Write integration test for launch + continue + exit in `tests/integration_tests.rs`

### Implementation for User Story 1

- [X] T024 [P] Implement `DebugEngine` orchestrator in `crates/rde-core/src/engine.rs` (new, run loop, handle_command)
- [X] T025 [P] Implement `Thread` struct and thread tracking in `crates/rde-core/src/lib.rs`
- [X] T026 Implement `RegisterContext` struct in `crates/rde-core/src/lib.rs`
- [X] T027 Implement `GetThreadContext` / `SetThreadContext` wrappers in `crates/rde-win32/src/thread.rs`
- [X] T028 Implement single-step via Trap flag in `crates/rde-win32/src/thread.rs`
- [X] T029 Implement `SuspendThread` / `ResumeThread` wrappers in `crates/rde-win32/src/thread.rs`
- [X] T030 Implement `ReadProcessMemory` / `WriteProcessMemory` wrappers in `crates/rde-win32/src/memory.rs`
- [X] T031 Implement `Breakpoint` struct and `BreakpointManager` in `crates/rde-breakpoint/src/lib.rs`
- [X] T032 Implement `BreakpointState` enum (Enabled, Disabled, Pending) in `crates/rde-breakpoint/src/lib.rs`
- [X] T033 Implement INT3 breakpoint set/remove (save original byte, write 0xCC) in `crates/rde-breakpoint/src/lib.rs`
- [X] T034 Implement breakpoint hit handler (restore original byte, decrement RIP, set Trap flag, step, reinstall 0xCC) in `crates/rde-breakpoint/src/hit_handler.rs`
- [X] T035 Integrate `BreakpointManager` with `DebugEngine` in `crates/rde-core/src/engine.rs`
- [X] T036 Implement runtime breakpoint injection via `SuspendThread` + `WriteProcessMemory` + `ResumeThread` in `crates/rde-core/src/engine.rs`
- [X] T037 Handle `CREATE_THREAD_DEBUG_EVENT` / `EXIT_THREAD_DEBUG_EVENT` in `crates/rde-win32/src/debug_loop.rs`
- [X] T038 Handle `LOAD_DLL_DEBUG_EVENT` / `UNLOAD_DLL_DEBUG_EVENT` in `crates/rde-win32/src/module.rs`
- [X] T039 Handle `EXIT_PROCESS_DEBUG_EVENT` with graceful session teardown in `crates/rde-core/src/engine.rs`
- [X] T040 Wire debug loop thread to send `EngineEvent` via channel in `crates/rde-win32/src/debug_loop.rs`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently.

---

## Phase 4: User Story 2 - REPL Interativo Responsivo (Priority: P2)

**Goal**: Interactive CLI that remains responsive (< 100ms) while target is running.

**Independent Test**: Start a debug session, run `continue`, then verify REPL accepts and processes `break`, `regs`, and `quit` commands without blocking.

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T041 [P] Write unit tests for command parser in `crates/rde-repl/tests/parser_tests.rs`
- [ ] T042 Write integration test for REPL command dispatch in `tests/integration_tests.rs`

### Implementation for User Story 2

- [X] T043 [P] Implement hand-written command tokenizer in `crates/rde-repl/src/parser.rs`
- [X] T044 [P] Implement command parser for all MVP commands in `crates/rde-repl/src/parser.rs`
- [X] T045 Implement `ParseError` type with helpful messages in `crates/rde-repl/src/parser.rs`
- [X] T046 Implement REPL async loop with `tokio::sync::mpsc` in `crates/rde-repl/src/lib.rs`
- [X] T047 Implement command executor / dispatch in `crates/rde-repl/src/executor.rs`
- [X] T048 Connect REPL `command_tx` to `DebugEngine` `command_rx` in `crates/rde-cli/src/main.rs`
- [X] T049 Connect `DebugEngine` `event_tx` to REPL `event_rx` in `crates/rde-cli/src/main.rs`
- [X] T050 Implement basic output formatter (strings) in `crates/rde-repl/src/formatter.rs`
- [X] T051 Implement `rde-cli` binary entry point with `clap` in `crates/rde-cli/src/main.rs`
- [X] T052 Add graceful shutdown on `Quit` command (close handles, join threads) in `crates/rde-cli/src/main.rs`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently.

---

## Phase 5: User Story 3 - Inspeção de Estado do Programa (Priority: P3)

**Goal**: Register display, memory read/write, stack trace with symbol resolution.

**Independent Test**: Pause at a known breakpoint, verify `regs` shows correct RIP, `x` reads memory, `bt` shows demangled Rust function names.

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T053 [P] Write snapshot test for register formatter in `crates/rde-repl/tests/snapshots/` using `insta`
- [ ] T054 [P] Write snapshot test for memory hex dump formatter in `crates/rde-repl/tests/snapshots/` using `insta`
- [ ] T055 Write integration test for stack trace with symbols in `tests/integration_tests.rs`

### Implementation for User Story 3

- [X] T056 [P] Implement register pretty-print formatter in `crates/rde-repl/src/formatter.rs`
- [X] T057 [P] Implement memory hex dump formatter (hex + ASCII) in `crates/rde-repl/src/formatter.rs`
- [ ] T058 Implement `VirtualQueryEx` wrapper for memory region info in `crates/rde-win32/src/memory.rs`
- [ ] T059 Implement memory write safety check (confirmation for executable regions or > 8 bytes) in `crates/rde-core/src/engine.rs`
- [ ] T060 Implement `SymbolEngine` trait in `crates/rde-symbols/src/lib.rs`
- [ ] T061 Implement `DbgHelpLoader` with `libloading` in `crates/rde-symbols/src/dbghelp.rs`
- [ ] T062 Implement `SymInitialize` / `SymCleanup` wrappers in `crates/rde-symbols/src/dbghelp.rs`
- [ ] T063 Implement `SymFromAddr` wrapper for symbol resolution in `crates/rde-symbols/src/dbghelp.rs`
- [ ] T064 Implement `SymGetLineFromAddr64` wrapper for source line info in `crates/rde-symbols/src/dbghelp.rs`
- [ ] T065 Implement `StackWalk64` wrapper in `crates/rde-symbols/src/dbghelp.rs`
- [ ] T066 Implement `rustc_demangle` wrapper in `crates/rde-symbols/src/demangler.rs`
- [ ] T067 Implement stack trace builder (walk + symbol resolve + demangle) in `crates/rde-symbols/src/lib.rs`
- [X] T068 Implement stack trace formatter in `crates/rde-repl/src/formatter.rs`
- [X] T069 Integrate `SymbolEngine` with `DebugEngine` in `crates/rde-core/src/engine.rs`
- [X] T070 Handle module load events to auto-register symbols in `crates/rde-core/src/engine.rs`
- [ ] T071 Implement `Module` tracking and `info modules` formatter in `crates/rde-repl/src/formatter.rs`
- [ ] T072 Implement `info threads` formatter in `crates/rde-repl/src/formatter.rs`
- [ ] T073 Integrate `capstone` for disassembly output in `crates/rde-symbols/src/lib.rs`
- [ ] T074 Implement `disas` command formatter in `crates/rde-repl/src/formatter.rs`

**Checkpoint**: All user stories should now be independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T075 [P] Add `///` doc comments and usage examples to all `pub` items in `crates/rde-core/src/`
- [X] T076 [P] Add `///` doc comments and safety proofs to all `unsafe` blocks in `crates/rde-win32/src/`
- [X] T077 [P] Create GitHub issues tracking each `unsafe` usage per Constitution VI
- [ ] T078 Add `.github/workflows/ci.yml` for `windows-latest` runner (build + test)
- [ ] T079 Write `docs/win32-debug-api.md` mapping each used API to its Rust wrapper
- [ ] T080 Run end-to-end integration test: full debug session from launch to exit
- [ ] T081 Run 10-minute stress test with repeated breakpoints to verify no resource leaks
- [ ] T082 Verify REPL responsiveness benchmark (< 100ms) under target execution load
- [ ] T083 Update `README.md` with build instructions and architecture diagram
- [X] T084 Review all error messages for clarity and user-friendliness in `crates/rde-repl/src/formatter.rs`
- [X] T085 Ensure all `tracing` spans are present on event-handling code paths per Constitution observability gate

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) — Depends on US1 for functional backend but REPL can be built in parallel against mock backend
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) — Depends on US1 for paused-state access; symbols can be built independently

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Create mock DebugBackend using mockall in crates/rde-core/tests/mock_backend.rs"
Task: "Write unit test for DebugEngine state transitions in crates/rde-core/tests/engine_tests.rs"
Task: "Write integration test for launch + continue + exit in tests/integration_tests.rs"

# Launch all backend implementations together:
Task: "Implement GetThreadContext / SetThreadContext wrappers in crates/rde-win32/src/thread.rs"
Task: "Implement ReadProcessMemory / WriteProcessMemory wrappers in crates/rde-win32/src/memory.rs"
Task: "Implement Breakpoint struct and BreakpointManager in crates/rde-breakpoint/src/lib.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (backend + breakpoints)
   - Developer B: User Story 2 (REPL + parser)
   - Developer C: User Story 3 (symbols + formatting)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
