# Tasks: Big Test Plan — rde-cli with Binary Tree Debuggee

**Input**: Design documents from `/specs/006-big-test-plan/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Organization**: Tasks implement manual test circuits (non-atomic, stateful sessions) in `test_cases/manual/`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure for the debuggee

- [ ] T001 Create `examples/rust_app_example/Cargo.toml` with project metadata and dependencies (std only)
- [ ] T002 [P] Create directory structure: `examples/rust_app_example/src/`, `test_cases/manual/`
- [ ] T003 Add `rust_app_example` as workspace member in root `Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY circuits can be written

**⚠️ CRITICAL**: No circuit writing can begin until this phase is complete

- [ ] T004 [P] Implement `TreeNode` struct with `value`, `left`, `right`, `height` in `examples/rust_app_example/src/main.rs`
- [ ] T005 [P] Implement `BinaryTree` struct with `root` and basic methods (`new`, `insert`, `search`, `delete`, `find_min`, `find_max`, `height`, `size`, `inorder_traversal`, `preorder_traversal`, `postorder_traversal`, `is_empty`, `clear`) in `examples/rust_app_example/src/main.rs`
- [ ] T006 [P] Implement `rotate_left` and `rotate_right` in `examples/rust_app_example/src/main.rs`
- [ ] T007 Implement demo scenario dispatcher (`--demo <name>`) with `InsertSequence`, `SearchMiss`, `DeleteRebalance`, `FullTraversal`, `StressTest` in `examples/rust_app_example/src/main.rs`
- [ ] T008 Compile `rust_app_example` and verify it runs all demo scenarios: `cargo run -p rust_app_example -- --demo insert-sequence`

**Checkpoint**: Foundation ready — debuggee compiles and runs. Circuit writing can now begin in parallel.

---

## Phase 3: User Story 1 — Launch & Session Initialization (Priority: P1) 🎯 MVP

**Goal**: Create manual test circuits for launch and session management

**Independent Test**: Execute CIRCUITO-01 through CIRCUITO-05 from `test_cases/manual/CIRCUITOS-LAUNCH-E-SESSAO.md`

### Implementation for User Story 1

- [ ] T009 [P] [US1] Create CIRCUITO-01: Full session cycle (launch → break → inspect → continue → exit) in `test_cases/manual/CIRCUITOS-LAUNCH-E-SESSAO.md`
- [ ] T010 [P] [US1] Create CIRCUITO-02: Multiple launches and quits in sequence in `test_cases/manual/CIRCUITOS-LAUNCH-E-SESSAO.md`
- [ ] T011 [P] [US1] Create CIRCUITO-03: 100-insert stress with dynamic breakpoints in `test_cases/manual/CIRCUITOS-LAUNCH-E-SESSAO.md`
- [ ] T012 [P] [US1] Create CIRCUITO-04: Cargo debug with stale check in `test_cases/manual/CIRCUITOS-LAUNCH-E-SESSAO.md`
- [ ] T013 [US1] Create CIRCUITO-05: Attach to running process in `test_cases/manual/CIRCUITOS-LAUNCH-E-SESSAO.md`

**Checkpoint**: CIRCUITOS-LAUNCH-E-SESSAO.md complete with 5 stateful circuits

---

## Phase 4: User Story 2 — Breakpoint Management (Priority: P1)

**Goal**: Create manual test circuits for breakpoint creation, deletion, and dynamic manipulation

**Independent Test**: Execute CIRCUITO-10 through CIRCUITO-14 from `test_cases/manual/CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md`

### Implementation for User Story 2

- [ ] T014 [P] [US2] Create CIRCUITO-10: Marathon breakpoints (create, hit, delete, recreate ×10) in `test_cases/manual/CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md`
- [ ] T015 [P] [US2] Create CIRCUITO-11: Dynamic breakpoint add during execution in `test_cases/manual/CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md`
- [ ] T016 [P] [US2] Create CIRCUITO-12: Duplicate breakpoints and selective removal in `test_cases/manual/CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md`
- [ ] T017 [P] [US2] Create CIRCUITO-13: System initial breakpoint followed by user breakpoints in `test_cases/manual/CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md`
- [ ] T018 [US2] Create CIRCUITO-14: Breakpoint on every tree function (full coverage) in `test_cases/manual/CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md`

**Checkpoint**: CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md complete with 5 stateful circuits

---

## Phase 5: User Story 3 — Execution Control Flow (Priority: P1)

**Goal**: Create manual test circuits for continue, step into, step over, step out across recursion depths

**Independent Test**: Execute CIRCUITO-20 through CIRCUITO-24 from `test_cases/manual/CIRCUITOS-EXECUCAO-E-NAVEGACAO.md`

### Implementation for User Story 3

- [ ] T019 [P] [US3] Create CIRCUITO-20: Complete descent — step into all tree levels (insert-sequence) in `test_cases/manual/CIRCUITOS-EXECUCAO-E-NAVEGACAO.md`
- [ ] T020 [P] [US3] Create CIRCUITO-21: Up and down — step out and step into alternated in `test_cases/manual/CIRCUITOS-EXECUCAO-E-NAVEGACAO.md`
- [ ] T021 [P] [US3] Create CIRCUITO-22: Complete traversal with step at each node (inorder/preorder/postorder) in `test_cases/manual/CIRCUITOS-EXECUCAO-E-NAVEGACAO.md`
- [ ] T022 [P] [US3] Create CIRCUITO-23: Delete with three different paths (leaf, one child, two children) in `test_cases/manual/CIRCUITOS-EXECUCAO-E-NAVEGACAO.md`
- [ ] T023 [US3] Create CIRCUITO-24: Manual rotation with structure inspection in `test_cases/manual/CIRCUITOS-EXECUCAO-E-NAVEGACAO.md`

**Checkpoint**: CIRCUITOS-EXECUCAO-E-NAVEGACAO.md complete with 5 stateful circuits

---

## Phase 6: User Story 4 — Variable & State Inspection (Priority: P1)

**Goal**: Create manual test circuits for print, vars, regs, bt across all function contexts

**Independent Test**: Execute CIRCUITO-30 through CIRCUITO-34 from `test_cases/manual/CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md`

### Implementation for User Story 4

- [ ] T024 [P] [US4] Create CIRCUITO-30: Complete insertion inspection — from None to 3-level tree in `test_cases/manual/CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md`
- [ ] T025 [P] [US4] Create CIRCUITO-31: Pretty print with dynamic config changes during session in `test_cases/manual/CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md`
- [ ] T026 [P] [US4] Create CIRCUITO-32: Vars, regs, bt at each recursion frame in `test_cases/manual/CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md`
- [ ] T027 [P] [US4] Create CIRCUITO-33: Traversal with Vec result pretty print (growing vector) in `test_cases/manual/CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md`
- [ ] T028 [US4] Create CIRCUITO-34: Multiple type inspection in same session (i32, Option, Vec, String, bool, Result, tuple) in `test_cases/manual/CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md`

**Checkpoint**: CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md complete with 5 stateful circuits

---

## Phase 7: User Story 5 — Pretty Printing of Rust Types (Priority: P2)

**Goal**: Pretty printing circuits are covered within US4 circuits (C31, C33, C34). No separate file needed.

**Independent Test**: Execute CIRCUITO-31, CIRCUITO-33, CIRCUITO-34

---

## Phase 8: User Story 6 — Tree Navigation & Function Coverage (Priority: P1)

**Goal**: Tree navigation circuits are covered within US3 circuits (C20, C22, C23, C24). No separate file needed.

**Independent Test**: Execute CIRCUITO-20, CIRCUITO-22, CIRCUITO-23, CIRCUITO-24

---

## Phase 9: User Story 7 — REPL Runtime Interaction (Priority: P2)

**Goal**: REPL runtime circuits are covered within US4 and US5 circuits (C31, C40, C45). No separate file needed.

**Independent Test**: Execute CIRCUITO-31, CIRCUITO-40, CIRCUITO-45

---

## Phase 10: User Story 8 — Thread, Module & Task Inspection (Priority: P2)

**Goal**: Create manual test circuit for threaded debuggee inspection

**Independent Test**: Execute CIRCUITO-43 from `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`

---

## Phase 11: User Story 9 — Memory & Disassembly Inspection (Priority: P2)

**Goal**: Create manual test circuits for memory examine and disassembly

**Independent Test**: Execute CIRCUITO-41 and CIRCUITO-42 from `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`

---

## Phase 12: User Story 10 — End-to-End Demo Scenarios (Priority: P1)

**Goal**: E2E circuits are covered within US1 circuits (C01, C03). No separate file needed.

**Independent Test**: Execute CIRCUITO-01, CIRCUITO-03

---

## Phase 13: Stress, Resilience & Error Handling

**Goal**: Create manual test circuits for stress testing and error recovery

**Independent Test**: Execute CIRCUITO-40, CIRCUITO-44, CIRCUITO-45 from `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`

### Implementation for Stress & Error Handling

- [ ] T029 [P] Create CIRCUITO-40: REPL marathon — 50 commands in sequence without quit in `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`
- [ ] T030 [P] Create CIRCUITO-41: Memory examine at all important addresses (code, stack, heap) in `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`
- [ ] T031 [P] Create CIRCUITO-42: Auto-disassembly and manual disassembly alternated in `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`
- [ ] T032 [P] Create CIRCUITO-43: Threaded debuggee — complete multi-thread inspection in `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`
- [ ] T033 [P] Create CIRCUITO-44: Stress test — 100 hits without stopping in `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`
- [ ] T034 [P] Create CIRCUITO-45: Session with invalid commands interleaved in `test_cases/manual/CIRCUITOS-REPL-MEMORIA-E-STRESS.md`

**Checkpoint**: CIRCUITOS-REPL-MEMORIA-E-STRESS.md complete with 6 stateful circuits

---

## Phase 14: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, documentation, and index

- [ ] T035 [P] Create `test_cases/manual/README.md` with circuit index and execution guide
- [ ] T036 [P] Validate that all 290 TCs from spec.md are covered across the 25 circuits
- [ ] T037 [P] Create golden path snapshot template for manual capture in `test_data/golden_paths/006-big-test-plan-template.txt`
- [ ] T038 Verify `rust_app_example` builds in release and debug profiles without warnings
- [ ] T039 [P] Review and normalize command formatting across all circuit files
- [ ] T040 Update `docs/quickstart.md` with debuggee build instructions

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all circuit writing
- **User Stories (Phase 3-12)**: All depend on Foundational phase completion
  - Can proceed sequentially in priority order (P1 → P2)
  - Or in parallel if team capacity allows
- **Stress & Error Handling (Phase 13)**: Depends on all user story phases
- **Polish (Phase 14)**: Depends on all circuit files being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational. No dependencies on other stories.
- **US2 (P1)**: Can start after Foundational. No dependencies on other stories.
- **US3 (P1)**: Can start after Foundational. No dependencies on other stories.
- **US4 (P1)**: Can start after Foundational. No dependencies on other stories.
- **US5 (P2)**: Covered within US4 circuits.
- **US6 (P1)**: Covered within US3 circuits.
- **US7 (P2)**: Covered within US4 and Stress circuits.
- **US8 (P2)**: Covered within Stress circuits.
- **US9 (P2)**: Covered within Stress circuits.
- **US10 (P1)**: Covered within US1 circuits.

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational is done, all user story circuit files can be written in parallel
- Within each circuit file, individual circuits marked [P] can be written in parallel
- Stress circuits and Polish tasks can run in parallel

---

## Implementation Strategy

### MVP First (US1 + US2 + US3 + US4)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (debuggee)
3. Complete Phase 3: US1 — Launch circuits (5 circuits)
4. Complete Phase 4: US2 — Breakpoint circuits (5 circuits)
5. Complete Phase 5: US3 — Execution control circuits (5 circuits)
6. Complete Phase 6: US4 — Inspection circuits (5 circuits)
7. **STOP and VALIDATE**: Execute CIRCUITO-01, CIRCUITO-10, CIRCUITO-20, CIRCUITO-30 manually
8. This gives a working manual test suite for core debugger features

### Incremental Delivery

1. Setup + Foundational → Debuggee ready
2. Add US1 circuits → Test launch and session
3. Add US2 circuits → Test breakpoints
4. Add US3 circuits → Test execution control
5. Add US4 circuits → Test inspection
6. Add Stress circuits → Test resilience
7. Polish → README and golden paths

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Writer A: US1 circuits (Launch)
   - Writer B: US2 circuits (Breakpoints)
   - Writer C: US3 circuits (Execution)
   - Writer D: US4 circuits (Inspection)
   - Writer E: Stress circuits (Resilience)
3. Review and integrate all circuit files

---

## Notes

- [P] tasks = different circuit files or sections, no dependencies
- [Story] label maps task to specific user story for traceability
- Each circuit is a **stateful session** — commands must be executed in sequence without quitting
- Circuits are designed for **manual execution** — copy commands from the .md files and paste into the REPL
- If a circuit fails, document the exact step and command for bug investigation
- Commit after each phase completion
- Stop at any checkpoint to validate circuits by executing them manually
