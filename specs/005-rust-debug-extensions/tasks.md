# Tasks: Rust Debug Extensions

**Input**: Design documents from `/specs/005-rust-debug-extensions/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Test tasks included per Constitution "Test-First" requirement and spec test scenarios.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Workspace crates: `crates/<crate-name>/`
- Integration tests: `tests/integration/`
- Unit tests: within each crate under `src/` or `tests/`
- Golden paths: `test_data/golden_paths/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and crate scaffolding

- [x] T001 Create `crates/rde-pretty-print/` directory with `Cargo.toml` and `src/lib.rs`
- [x] T002 Create `crates/rde-tokio/` directory with `Cargo.toml` and `src/lib.rs`
- [x] T003 Create `crates/rde-cargo/` directory with `Cargo.toml` and `src/lib.rs`
- [x] T004 [P] Add `rde-pretty-print`, `rde-tokio`, `rde-cargo` to workspace `Cargo.toml` members list
- [x] T005 [P] Configure crate dependencies: `rde-pretty-print` depends on `rde-core`, `rde-win32`, `rde-symbols`, `tracing`
- [x] T006 [P] Configure crate dependencies: `rde-tokio` depends on `rde-core`, `rde-win32`, `rde-symbols`, `tracing`
- [x] T007 [P] Configure crate dependencies: `rde-cargo` depends on `serde`, `serde_json`, `tokio::process`, `tracing`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core protocol and abstraction layers that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T008 Extend `EngineRequest` enum in `crates/rde-core/src/protocol.rs` with `Print { frame_id: u64, expression: String }`, `ListTasks`, `CargoLaunch { manifest_path: PathBuf, package: Option<String>, target: Option<String>, profile: String, features: Vec<String> }`
- [x] T009 Extend `EngineResponse` enum in `crates/rde-core/src/protocol.rs` with `PrettyValue(PrettyValue)`, `TaskList(Vec<AsyncTask>)`, `CargoLaunchResult(Result<ProcessId, CargoError>)`
- [x] T010 Define shared `PrettyValue` enum in `crates/rde-core/src/value.rs` with variants `Scalar`, `Enum`, `Sequence`, `Map`, `Raw`, `Truncated`
- [x] T011 Define shared `TaskState` enum in `crates/rde-core/src/task.rs` with variants `Running`, `Idle`, `Sleeping`, `Completed`
- [x] T012 Define shared `AsyncTask` struct in `crates/rde-core/src/task.rs` with fields `task_id: u64`, `state: TaskState`, `function_name: Option<String>`, `runtime_thread_id: Option<u32>`
- [x] T013 Define `CargoTarget` and `CargoTargetKind` structs/enums in `crates/rde-core/src/cargo.rs`
- [x] T014 Create `MemoryReader` trait in `crates/rde-win32/src/memory.rs` (or extend existing) providing `read_bytes(address: usize, len: usize) -> Result<Vec<u8>>` for safe target memory access
- [x] T015 Implement `FormatBudget` struct in `crates/rde-pretty-print/src/budget.rs` with `max_depth`, `max_elements`, `max_bytes` and decrement methods
- [x] T016 Implement `PrinterRegistry` in `crates/rde-pretty-print/src/registry.rs` with `register(type_name: &str, printer: Box<dyn PrettyPrinter>)` and `lookup(type_name: &str) -> Option<&dyn PrettyPrinter>`
- [x] T017 Define `PrettyPrinter` trait in `crates/rde-pretty-print/src/lib.rs` with `format(&self, reader: &dyn MemoryReader, address: usize, budget: &mut FormatBudget) -> PrettyValue`
- [x] T018 Define `CargoError` enum in `crates/rde-cargo/src/lib.rs` covering `MetadataFailure`, `BuildFailure`, `TargetNotFound`, `InvalidManifest`

**Checkpoint**: Foundation ready — `EngineRequest`/`EngineResponse` protocol extended, shared types defined, `MemoryReader` available, registry and budget scaffolding complete. User story implementation can now begin.

---

## Phase 3: User Story 1 - Visualizar valores Rust complexos com pretty printers (Priority: P1) 🎯 MVP

**Goal**: Pretty-print standard Rust types (`Option`, `Vec`, `String`, `HashMap`, `Result`) from target memory into human-readable representations via REPL commands `print` and `vars`.

**Independent Test**: Set a breakpoint in a debuggee using `Option<i32>`, `Vec<String>`, and `HashMap<u32, u32>`. Run `print` or `vars` in REPL and verify output shows `Some(42)`, `[10, 20, 30]`, and `{1: "one", 2: "two"}` instead of raw memory layouts.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T019 [P] [US1] Unit test for `FormatBudget` decrement and saturation in `crates/rde-pretty-print/src/budget.rs`
- [x] T020 [P] [US1] Unit test for `PrinterRegistry` lookup and priority in `crates/rde-pretty-print/src/registry.rs`
- [x] T021 [P] [US1] Snapshot test for `Option<i32>::Some(42)` pretty-print output in `crates/rde-pretty-print/tests/snapshots/option.snap`
- [x] T022 [P] [US1] Snapshot test for `Vec<i32>` pretty-print output (including truncation) in `crates/rde-pretty-print/tests/snapshots/vec.snap`
- [x] T023 [P] [US1] Snapshot test for `String` pretty-print output in `crates/rde-pretty-print/tests/snapshots/string.snap`
- [x] T024 [P] [US1] Snapshot test for nested `Option<Vec<Option<String>>>` with depth limit in `crates/rde-pretty-print/tests/snapshots/nested.snap`
- [x] T025 [US1] Integration test: mock `MemoryReader` returning known bytes for a `Vec<u8>` layout, assert `PrettyValue::Sequence` produced in `tests/integration/pretty_print_mock.rs`

### Implementation for User Story 1

- [x] T026 [P] [US1] Implement `OptionPrinter` in `crates/rde-pretty-print/src/printers/option.rs` reading discriminant and payload via `MemoryReader`
- [x] T027 [P] [US1] Implement `VecPrinter` in `crates/rde-pretty-print/src/printers/vec.rs` reading `(ptr, len, cap)` triple and iterating elements with `FormatBudget` limit
- [x] T028 [P] [US1] Implement `StringPrinter` in `crates/rde-pretty-print/src/printers/string.rs` reading `Vec<u8>` inner data and decoding UTF-8
- [x] T029 [P] [US1] Implement `HashMapPrinter` in `crates/rde-pretty-print/src/printers/hashmap.rs` printing summary (`len` / `capacity`) with optional deep iteration up to budget
- [x] T030 [P] [US1] Implement `ResultPrinter` in `crates/rde-pretty-print/src/printers/result.rs` reading discriminant and `Ok`/`Err` payload
- [x] T031 [US1] Register all built-in printers in `PrinterRegistry` inside `crates/rde-pretty-print/src/lib.rs` at initialization
- [x] T032 [US1] Implement recursive dispatch in `crates/rde-pretty-print/src/lib.rs`: resolve element type via `rde-symbols`, look up printer, recurse with decremented `FormatBudget`
- [x] T033 [US1] Add REPL command parser for `print <expr>` and `vars` in `crates/rde-repl/src/commands.rs`
- [x] T034 [US1] Wire REPL `print`/`vars` commands to emit `EngineRequest::Print` via channels in `crates/rde-repl/src/executor.rs`
- [x] T035 [US1] Handle `EngineRequest::Print` in debug loop/engine: read variable address from stack frame, resolve type name via `rde-symbols`, invoke `rde-pretty-print`, return `EngineResponse::PrettyValue` in `crates/rde-core/src/engine.rs`
- [ ] T036 [US1] Format `PrettyValue` for REPL display in `crates/rde-repl/src/display.rs` (human-readable output with truncation indicators)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Running `print my_vec` at a breakpoint shows `[10, 20, 30]`.

---

## Phase 4: User Story 2 - Inspecionar tarefas assíncronas do Tokio (Priority: P2)

**Goal**: Detect Tokio runtime in the debuggee and list async tasks with their states and associated function names via REPL command `tasks`.

**Independent Test**: Spawn 3 Tokio tasks in a debuggee, stop at a breakpoint, run `tasks` in REPL, and verify a table with task IDs, states (`running`/`idle`/`sleeping`), and function names is displayed.

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T037 [P] [US2] Unit test for `TaskState` parsing from raw header bits in `crates/rde-tokio/src/task.rs`
- [x] T038 [P] [US2] Unit test for `AsyncTask` formatting/display in `crates/rde-tokio/src/task.rs`
- [x] T039 [US2] Integration test with mock memory mimicking Tokio 1.x runtime header: assert scanner finds 2 tasks in `tests/integration/tokio_mock.rs`

### Implementation for User Story 2

- [x] T040 [US2] Implement Tokio runtime signature / vtable scanner in `crates/rde-tokio/src/scanner.rs` to locate runtime global in target memory
- [x] T041 [US2] Implement task list walker in `crates/rde-tokio/src/scanner.rs` following Tokio scheduler internal links to enumerate active tasks
- [x] T042 [US2] Implement task state decoder in `crates/rde-tokio/src/task.rs` mapping Tokio header bit patterns to `TaskState` enum
- [x] T043 [US2] Resolve task function names via `rde-symbols` (`SymFromAddr` on task vtable / poll function pointer) in `crates/rde-tokio/src/scanner.rs`
- [x] T044 [US2] Add REPL command parser for `tasks` in `crates/rde-repl/src/commands.rs`
- [x] T045 [US2] Wire REPL `tasks` command to emit `EngineRequest::ListTasks` via channels in `crates/rde-repl/src/executor.rs`
- [x] T046 [US2] Handle `EngineRequest::ListTasks` in debug loop/engine: invoke `rde-tokio` scanner when process is stopped, return `EngineResponse::TaskList` in `crates/rde-core/src/engine.rs`
- [ ] T047 [US2] Format `TaskList` as table for REPL display in `crates/rde-repl/src/display.rs`
- [x] T048 [US2] Handle "no Tokio runtime detected" gracefully: return informative message when scanner finds no signature in `crates/rde-tokio/src/scanner.rs`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. `tasks` command displays Tokio task table; pretty printers remain functional.

---

## Phase 5: User Story 3 - Integração com Cargo para debug facilitado (Priority: P3)

**Goal**: Launch debug sessions directly from a Cargo project directory, resolving the binary target automatically and triggering `cargo build` when the artifact is stale.

**Independent Test**: In a directory with a valid `Cargo.toml`, run `rde cargo debug`. Verify the correct binary from `target/debug/` is launched, and that `cargo build` runs automatically if sources are newer than the artifact.

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T049 [P] [US3] Unit test for `CargoTarget` path resolution logic in `crates/rde-cargo/src/target.rs`
- [x] T050 [P] [US3] Unit test for artifact staleness check (mtime comparison) in `crates/rde-cargo/src/build.rs`
- [x] T051 [US3] Unit test for `cargo metadata` JSON parsing into `CargoProject` in `crates/rde-cargo/src/metadata.rs`
- [x] T052 [US3] Integration test: mock `cargo metadata` output and assert correct target resolution in `tests/integration/cargo_target.rs`

### Implementation for User Story 3

- [x] T053 [US3] Implement `cargo metadata` invocation and JSON parsing in `crates/rde-cargo/src/metadata.rs` using `serde`
- [x] T054 [US3] Implement `CargoTarget` resolution from metadata in `crates/rde-cargo/src/target.rs` (package → target → profile → artifact path)
- [x] T055 [US3] Implement artifact staleness check in `crates/rde-cargo/src/build.rs`: compare artifact mtime against newest source file mtime in package `src/` directory
- [x] T056 [US3] Implement `cargo build` subprocess spawn in `crates/rde-cargo/src/build.rs` using `tokio::process::Command`, capturing stdout/stderr
- [x] T057 [US3] Implement async wait for `cargo build` completion with REPL-responsive polling in `crates/rde-cargo/src/build.rs`
- [x] T058 [US3] Add CLI subcommand parser for `cargo debug [opts]` in `crates/rde-cli/src/args.rs`
- [x] T059 [US3] Wire `cargo debug` CLI flow through `rde-cargo` → resolve target → build if stale → hand binary path to `rde-core` launcher in `crates/rde-cli/src/main.rs`
- [x] T060 [US3] Propagate `cargo build` errors (compilation failures) to user with stderr output in `crates/rde-cargo/src/build.rs`
- [x] T061 [US3] Handle workspace packages: allow `--package <name>` to select correct manifest and target in `crates/rde-cargo/src/target.rs`

**Checkpoint**: All user stories should now be independently functional. `cargo debug` launches from Cargo projects; pretty printers and Tokio tasks work in the resulting debug session.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T062 [P] Generate golden path snapshot `test_data/golden_paths/005-rust-debug-extensions.txt` covering: breakpoint → `print vec` → `tasks` → `continue`
- [ ] T063 [P] Update `docs/contracts/win32-debug-api.md` with `ReadProcessMemory` pretty-print usage contract if not already present
- [ ] T064 Add `tracing` instrumentation to all new engine request/response handlers in `crates/rde-core/src/engine.rs`
- [ ] T065 Performance validation: benchmark `Vec<T>` pretty-print with 100 elements; verify <1s in `tests/perf/pretty_print_bench.rs`
- [ ] T066 Performance validation: benchmark Tokio task listing with 10 tasks; verify <2s in `tests/perf/tokio_tasks_bench.rs`
- [ ] T067 [P] Add `--raw` flag to `print` command in `crates/rde-repl/src/commands.rs` to bypass pretty printing
- [ ] T068 [P] Add `--limit <n>` flag to `print` command for overriding default element limit in `crates/rde-repl/src/commands.rs`
- [ ] T069 [P] Add `--depth <n>` flag to `print` command for overriding default recursion depth in `crates/rde-repl/src/commands.rs`
- [ ] T070 Validate `quickstart.md` steps manually against a sample Cargo + Tokio project
- [ ] T071 Run `cargo test --workspace` and fix any regressions in existing crates caused by new dependencies or protocol changes
- [ ] T072 Update `CLAUDE.md` workspace layout section to include `rde-pretty-print`, `rde-tokio`, `rde-cargo`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3–5)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) — Integrates with US1 via shared `EngineRequest`/`EngineResponse` protocol but is independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) — Independent pre-launch flow; does not depend on US1 or US2 at runtime

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Models/types before services/scanners
- Core crate implementation before REPL/CLI integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks (T001–T007) can run in parallel
- All Foundational type definitions (T008–T018) can run in parallel (different files, no runtime deps)
- US1 test writing (T019–T025) can run in parallel
- US1 printer implementations (T026–T030) can run in parallel (different files)
- US2 and US3 can be implemented in parallel once Foundational is done
- Polish tasks T062, T063, T067–T069 can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for FormatBudget in crates/rde-pretty-print/src/budget.rs"
Task: "Snapshot test for Option<i32> in crates/rde-pretty-print/tests/snapshots/option.snap"
Task: "Snapshot test for Vec<i32> in crates/rde-pretty-print/tests/snapshots/vec.snap"

# Launch all printer implementations together:
Task: "Implement OptionPrinter in crates/rde-pretty-print/src/printers/option.rs"
Task: "Implement VecPrinter in crates/rde-pretty-print/src/printers/vec.rs"
Task: "Implement StringPrinter in crates/rde-pretty-print/src/printers/string.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 (pretty printers for `Option`, `Vec`, `String`)
4. **STOP and VALIDATE**: Test `print` and `vars` commands independently
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
   - Developer A: User Story 1 (pretty printers)
   - Developer B: User Story 2 (Tokio tasks)
   - Developer C: User Story 3 (Cargo integration)
3. Stories complete and integrate independently via shared `EngineRequest`/`EngineResponse` protocol

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
