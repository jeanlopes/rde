# Tasks: Thread and Module Tracking

**Input**: Design documents from `/specs/002-thread-module-tracking/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Integration tests included per Constitution Test-First requirement. Tests are written alongside implementation tasks.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare test infrastructure and golden path artifacts for the feature.

- [ ] T001 Create golden path directory and scaffold for `002-thread-module-tracking` in `test_data/golden_paths/002-thread-module-tracking.txt`
- [ ] T002 [P] Add multi-threaded example program for integration tests in `examples/threaded_debuggee.rs`
- [ ] T003 [P] Add DLL-loading example program for integration tests in `examples/dll_debuggee.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core type system and debug loop changes that MUST be complete before ANY user story can be implemented.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T004 Add `SelectThread { id: ThreadId }` variant to `EngineCommand` in `crates/rde-core/src/events.rs`
- [ ] T005 Add `ModuleUnloaded { base: u64 }` variant to `EngineEvent` in `crates/rde-core/src/events.rs`
- [ ] T006 [P] Update `Session` struct to hold `threads: HashMap<ThreadId, Thread>`, `modules: HashMap<u64, Module>`, and `selected_thread: Option<ThreadId>` in `crates/rde-core/src/engine.rs`
- [ ] T007 [P] Update `DebugBackend` trait to accept `thread_handle: Option<RawHandle>` for cached handle operations in `crates/rde-core/src/lib.rs`
- [ ] T008 Extract and cache `hThread` from `CREATE_THREAD_DEBUG_EVENT` in `crates/rde-win32/src/debug_loop.rs`
- [ ] T009 Extract `lpBaseOfDll` from `LOAD_DLL_DEBUG_EVENT` in `crates/rde-win32/src/debug_loop.rs`
- [ ] T010 Add `UNLOAD_DLL_DEBUG_EVENT` handler to debug loop dispatch in `crates/rde-win32/src/debug_loop.rs`
- [ ] T011 [P] Add `CloseHandle` safety wrapper and thread handle cache helper in `crates/rde-win32/src/thread.rs`

**Checkpoint**: Foundation ready — `Session` has registries, debug loop extracts handles and module bases, and new event/command variants exist. User story implementation can now begin.

---

## Phase 3: User Story 1 - Visualizar e Selecionar Threads (Priority: P1) 🎯 MVP

**Goal**: The debugger maintains a live thread registry, allows listing threads, and uses a selected thread as the default context for `regs`, `bt`, and `step`.

**Independent Test**: Launch a multi-threaded program, run `threads`, verify all TIDs appear. Select a non-main thread with `thread <id>`, run `regs`, and verify the register values differ from the main thread.

### Implementation for User Story 1

- [ ] T012 [P] [US1] Implement `ThreadCreated` event handler: insert thread into registry with cached handle in `crates/rde-core/src/engine.rs`
- [ ] T013 [P] [US1] Implement `ThreadExited` event handler: mark thread as `Exited`, close cached handle, and reassign `selected_thread` to oldest remaining thread in `crates/rde-core/src/engine.rs`
- [ ] T014 [US1] Implement `ListThreads` command handler: format registry as table with state and selected indicator in `crates/rde-core/src/engine.rs`
- [ ] T015 [US1] Implement `SelectThread` command handler: validate thread exists and is not exited, update `selected_thread` in `crates/rde-core/src/engine.rs`
- [ ] T016 [US1] Update `StepInto`, `ReadRegisters`, and `Backtrace` to use `session.selected_thread` instead of hardcoded `0` in `crates/rde-core/src/engine.rs`
- [ ] T017 [P] [US1] Add `thread <id>` command parsing to REPL in `crates/rde-repl/src/parser.rs`
- [ ] T018 [US1] Display `ThreadCreated` and `ThreadExited` events in REPL output loop in `crates/rde-repl/src/lib.rs`
- [ ] T019 [P] [US1] Write integration test for thread listing and selection in `tests/integration_tests.rs`
- [ ] T020 [US1] Capture and write golden path snapshot for thread lifecycle events in `test_data/golden_paths/002-thread-module-tracking.txt`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. The MVP is complete.

---

## Phase 4: User Story 2 - Rastrear Módulos Carregados (Priority: P2)

**Goal**: The debugger maintains a live module registry, lists loaded DLLs with base addresses and sizes, and handles dynamic load/unload events.

**Independent Test**: Launch a program that calls `LoadLibrary`, run `modules`, verify the loaded DLL appears with correct base address. After `FreeLibrary`, verify the DLL is removed from the list.

### Implementation for User Story 2

- [ ] T021 [P] [US2] Implement `ModuleLoaded` event handler: resolve module name via `GetMappedFileNameW`, insert into registry with base and size in `crates/rde-core/src/engine.rs`
- [ ] T022 [P] [US2] Implement `ModuleUnloaded` event handler: remove module from registry by base address in `crates/rde-core/src/engine.rs`
- [ ] T023 [US2] Implement `ListModules` command handler: format registry as table with name, base, size, and symbol status in `crates/rde-core/src/engine.rs`
- [ ] T024 [US2] Implement `get_module_name` using `GetMappedFileNameW` and NT-to-DOS path translation in `crates/rde-win32/src/module.rs`
- [ ] T025 [US2] Implement `enumerate_modules` fallback using `EnumProcessModules` + `K32GetModuleInformation` in `crates/rde-win32/src/module.rs`
- [ ] T026 [US2] Update debug loop `LOAD_DLL_DEBUG_EVENT` dispatch to send real `base` and resolved `name` in `crates/rde-win32/src/debug_loop.rs`
- [ ] T027 [US2] Update debug loop `UNLOAD_DLL_DEBUG_EVENT` dispatch to send `ModuleUnloaded` with `lpBaseOfDll` in `crates/rde-win32/src/debug_loop.rs`
- [ ] T028 [US2] Display `ModuleLoaded` and `ModuleUnloaded` events in REPL output loop in `crates/rde-repl/src/lib.rs`
- [ ] T029 [P] [US2] Write integration test for module load/unload tracking in `tests/integration_tests.rs`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Modules are tracked and listed.

---

## Phase 5: User Story 3 - Integração com Resolução de Símbolos (Priority: P3)

**Goal**: When a DLL loads, the debugger notifies the symbol engine so that stack traces and breakpoints resolve names inside dynamically loaded modules.

**Independent Test**: Launch a program that loads a DLL with PDB symbols, trigger a stack trace after the DLL loads, and verify function names from the DLL appear resolved instead of raw addresses.

### Implementation for User Story 3

- [ ] T030 [US3] Add `SymLoadModuleEx` FFI declaration and loader symbol to `DbgHelpLoader` in `crates/rde-symbols/src/dbghelp.rs`
- [ ] T031 [US3] Implement `DbgHelpSymbolEngine::load_module` to call `SymLoadModuleEx` with base, size, and path in `crates/rde-symbols/src/dbghelp.rs`
- [ ] T032 [US3] Add `SymbolEngine::unload_module` trait method and implement via `SymUnloadModule64` in `crates/rde-symbols/src/dbghelp.rs`
- [ ] T033 [US3] Integrate symbol engine `load_module` call into `ModuleLoaded` handler in `crates/rde-core/src/engine.rs`
- [ ] T034 [US3] Integrate symbol engine `unload_module` call into `ModuleUnloaded` handler in `crates/rde-core/src/engine.rs`
- [ ] T035 [US3] Update `Module` registry entries to set `symbols_loaded = true` on successful `SymLoadModuleEx` in `crates/rde-core/src/engine.rs`
- [ ] T036 [P] [US3] Write integration test for symbol resolution after DLL load in `tests/integration_tests.rs`

**Checkpoint**: All user stories should now be independently functional. Symbol resolution works for dynamically loaded modules.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, observability, contracts, and final validation.

- [ ] T037 [P] Handle edge case: thread principal exits before process — ensure session stays active and fallback selection works in `crates/rde-core/src/engine.rs`
- [ ] T038 [P] Handle edge case: select non-existent or exited thread — return clear error to REPL in `crates/rde-core/src/engine.rs`
- [ ] T039 [P] Handle edge case: module loaded without symbols — display correctly in `ListModules` with `✗` symbol status in `crates/rde-core/src/engine.rs`
- [ ] T040 [P] Add `tracing` instrumentation to all new event handlers and command handlers in `crates/rde-core/src/engine.rs`
- [ ] T041 [P] Update `docs/contracts/win32-debug-api.md` with contracts for `OpenThread`, `GetMappedFileNameW`, `EnumProcessModules`, and `SymLoadModuleEx`
- [ ] T042 [P] Run full workspace test suite (`cargo test --workspace`) and fix regressions
- [ ] T043 [P] Validate quickstart.md commands against actual REPL output

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories.
- **User Stories (Phase 3–5)**: All depend on Foundational phase completion.
  - US1 (P1) → US2 (P2) → US3 (P3) recommended sequential order.
  - US2 and US3 can theoretically start in parallel with US1 if the engine event arms are written generically, but sequential reduces merge conflicts in `engine.rs`.
- **Polish (Phase 6)**: Depends on all user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2). No dependencies on other stories. This is the MVP.
- **User Story 2 (P2)**: Can start after Foundational (Phase 2). Reads the same `Session` struct as US1 but operates on `modules` instead of `threads`. Minimal coupling.
- **User Story 3 (P3)**: Can start after US2 is complete because it depends on `ModuleLoaded` events and the module registry being populated.

### Within Each User Story

- Models / registry fields before command handlers.
- Command handlers before REPL display updates.
- Core implementation before integration tests.
- Story complete before moving to next priority.

### Parallel Opportunities

- All Setup tasks (T001–T003) can run in parallel.
- All Foundational type changes (T004–T007) can run in parallel with debug loop extractions (T008–T010).
- Within US1: T012, T013, T017 can run in parallel.
- Within US2: T021, T022, T024, T025 can run in parallel.
- Within US3: T030, T031, T032 can run in parallel.
- All Polish tasks (T037–T043) can run in parallel.

---

## Parallel Example: User Story 1

```bash
# Launch registry handlers and REPL parser in parallel:
Task: "Implement ThreadCreated event handler in crates/rde-core/src/engine.rs"
Task: "Implement ThreadExited event handler in crates/rde-core/src/engine.rs"
Task: "Add thread <id> command parsing in crates/rde-repl/src/parser.rs"

# Then sequential dependency:
Task: "Implement ListThreads command handler in crates/rde-core/src/engine.rs"
Task: "Implement SelectThread command handler in crates/rde-core/src/engine.rs"
Task: "Update StepInto/ReadRegisters/Backtrace in crates/rde-core/src/engine.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test `threads`, `thread <id>`, `regs`, `bt`, `step` independently
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
   - Developer A: User Story 1 (engine registry + REPL)
   - Developer B: User Story 2 (module resolution + REPL)
   - Developer C: User Story 3 (symbol engine wiring)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (Red-Green-Refactor)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
