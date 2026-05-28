# Tasks: TUI Interface with Multi-Pane Layout

**Input**: Design documents from `/specs/004-tui-interface/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Test tasks included for golden path validation per plan.md recommendations.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the `rde-tui` crate and add workspace dependencies

- [x] T001 Create crate directory `crates/rde-tui/` with `Cargo.toml` referencing workspace dependencies and adding `ratatui` and `crossterm`
- [x] T002 [P] Register `rde-tui` in workspace `Cargo.toml` members list
- [x] T003 [P] Create `crates/rde-tui/src/lib.rs` with crate root and re-exports
- [x] T004 Create `crates/rde-tui/src/app.rs` with `TuiApp` struct stub and `run()` entry point

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Add structured `EngineEvent` variants (`Registers`, `Disassembly`, `BreakpointList`, `StackTrace`) to `crates/rde-core/src/events.rs`
- [x] T006 [P] Update `DebugEngine` in `crates/rde-core/src/engine.rs` to emit structured events after `ReadRegisters`, `Disassemble`, `Backtrace`, and breakpoint mutations
- [x] T007 Implement `DbgHelpSymbolEngine::walk_stack()` in `crates/rde-symbols/src/dbghelp.rs` using `StackWalk64`
- [x] T008 Create `crates/rde-tui/src/panes/mod.rs` with `Pane` trait, `PaneType` enum, and `PaneContext`
- [x] T009 [P] Create `crates/rde-tui/src/session_mirror.rs` with `SessionMirror` struct and incremental update methods from `EngineEvent`
- [x] T010 Create `crates/rde-tui/src/layout.rs` with `LayoutConfig`, `SplitDirection`, and default multi-pane layout constraints
- [x] T011 Implement TUI event loop in `crates/rde-tui/src/app.rs` integrating `crossterm` input events, terminal resize, and `EngineEvent` stream via `tokio::select!`

**Checkpoint**: Foundation ready — `rde-tui` compiles, event loop runs, and receives `EngineEvent`s; user story implementation can now begin in parallel

---

## Phase 3: User Story 1 — Visualização Multi-Pane durante Debug (Priority: P1) 🎯 MVP

**Goal**: Render source/asm, registers, stack, and breakpoints simultaneously in a single terminal window, updating automatically on debug state changes.

**Independent Test**: Launch `rde-cli --tui --target hello_debuggee.exe`, hit a breakpoint, and verify all four panes display correct data without manual refresh.

### Tests for User Story 1

- [x] T012 [P] [US1] Add golden path test script in `test_data/golden_paths/tui-session.txt` documenting expected render sequence after breakpoint hit
- [x] T013 [P] [US1] Add snapshot test in `crates/rde-tui/tests/pane_tests.rs` for `RegistersPane` render output using `insta`

### Implementation for User Story 1

- [x] T014 [P] [US1] Implement `SourceAsmPane` in `crates/rde-tui/src/panes/source_asm.rs` rendering `DisassemblyLine` list with current address highlight
- [x] T015 [P] [US1] Implement `RegistersPane` in `crates/rde-tui/src/panes/registers.rs` rendering `RegisterContext` as a two-column table
- [x] T016 [P] [US1] Implement `StackPane` in `crates/rde-tui/src/panes/stack.rs` rendering `Vec<StackFrame>` with symbol names and return addresses
- [x] T017 [P] [US1] Implement `BreakpointsPane` in `crates/rde-tui/src/panes/breakpoints.rs` rendering `Vec<Breakpoint>` with address, hit count, and state
- [x] T018 [US1] Implement layout rendering in `crates/rde-tui/src/app.rs` using `ratatui::Layout` constraints to divide terminal area into five pane regions
- [x] T019 [US1] Implement focus management in `crates/rde-tui/src/app.rs` ensuring exactly one pane has focus and visual indicator at all times
- [x] T020 [US1] Wire pane update logic in `crates/rde-tui/src/app.rs` so each pane receives data from `SessionMirror` on every `EngineEvent`
- [x] T021 [US1] Add `--tui` CLI flag to `crates/rde-cli/src/main.rs` routing to `rde_tui::run()` instead of `rde_repl::run()`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 — Interação via REPL Integrado (Priority: P2)

**Goal**: Provide an embedded REPL pane inside the TUI where users can type debugger commands and see results without leaving the multi-pane view.

**Independent Test**: Focus the REPL pane, type `break 0x140001000`, press Enter, and verify the command is sent to the engine and the response appears in the REPL history while other panes remain visible.

### Tests for User Story 2

- [x] T022 [P] [US2] Add unit test in `crates/rde-tui/tests/pane_tests.rs` verifying `ReplPane` command buffer accumulation and history append

### Implementation for User Story 2

- [x] T023 [US2] Implement `ReplPane` in `crates/rde-tui/src/panes/repl.rs` with command input line, scrollback history, and cursor handling
- [x] T024 [US2] Integrate `ReplPane` keyboard handling in `crates/rde-tui/src/app.rs` capturing alphanumeric input when REPL is focused and routing `Enter` to command dispatch
- [x] T025 [US2] Wire REPL command dispatch in `crates/rde-tui/src/app.rs` parsing input into `EngineCommand` and sending via `command_tx`
- [x] T026 [US2] Implement command history navigation (`↑`/`↓`) in `crates/rde-tui/src/panes/repl.rs`
- [x] T027 [US2] Display structured engine responses in REPL scrollback (registers, disassembly, stack trace) using `SessionMirror` data instead of raw `Output` strings
- [x] T028 [US2] Add error handling for invalid REPL commands showing clear inline messages without disrupting other panes

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 — Adaptação a Diferentes Tamanhos de Terminal (Priority: P3)

**Goal**: The TUI layout adapts gracefully to terminal resizing and enforces a minimum usable size.

**Independent Test**: Launch the TUI, progressively shrink the terminal window, and verify the interface remains legible down to 80×24; below that, a warning overlay appears.

### Tests for User Story 3

- [x] T029 [P] [US3] Add unit test in `crates/rde-tui/tests/pane_tests.rs` verifying `LayoutConfig` recalculates constraints for 80×24 and 120×40 terminals

### Implementation for User Story 3

- [x] T030 [US3] Handle `crossterm::event::Event::Resize` in `crates/rde-tui/src/app.rs` triggering `LayoutConfig` recalculation
- [x] T031 [US3] Implement minimum size check in `crates/rde-tui/src/app.rs` (80 cols × 24 rows) displaying a centered warning overlay when below threshold
- [x] T032 [US3] Add pane content truncation/elision in each pane widget (`source_asm.rs`, `registers.rs`, `stack.rs`, `breakpoints.rs`) to prevent overflow on small terminals
- [x] T033 [US3] Implement proportional pane redistribution in `crates/rde-tui/src/layout.rs` ensuring no pane collapses below a minimum usable height/width
- [x] T034 [US3] Preserve pane scroll positions across resize events in `crates/rde-tui/src/session_mirror.rs`

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T035 [P] Implement `StatusBar` widget in `crates/rde-tui/src/widgets/status_bar.rs` showing session state, selected thread, and current address
- [x] T036 [P] Implement `HelpBar` widget in `crates/rde-tui/src/widgets/help_bar.rs` showing context-sensitive keybindings for the focused pane
- [x] T037 Add F-key shortcuts (`F5`, `F9`, `F10`, `F11`) in `crates/rde-tui/src/app.rs` mapping to `EngineCommand::Continue`, `SetBreakpoint`, `StepOver`, `StepInto`
- [x] T038 Add `Ctrl+Q` quit handling and terminal cleanup (restore raw mode, clear screen) in `crates/rde-tui/src/app.rs`
- [x] T039 [P] Write golden path snapshot in `test_data/golden_paths/tui-session.txt` matching the expected event/render sequence
- [x] T040 [P] Update `docs/quickstart.md` with TUI-specific launch instructions and keybinding reference
- [x] T041 Verify `cargo test --workspace` passes with new `rde-tui` tests included
- [x] T042 Run `cargo clippy --workspace` and fix all warnings introduced by `rde-tui`

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
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) — Builds on US1 panes but ReplPane is independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) — Layout changes are orthogonal to pane content

### Within Each User Story

- Tests MUST be written and confirmed failing before implementation (where test tasks exist)
- Pane widgets within a story marked [P] can be implemented in parallel
- Core layout/integration tasks depend on pane completion
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational is done:
  - All four pane widgets in US1 (T014–T017) can be developed in parallel
  - US2 and US3 can start in parallel with each other and with US1 integration tasks
- All Polish tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all pane implementations for User Story 1 together:
Task: "Implement SourceAsmPane in crates/rde-tui/src/panes/source_asm.rs"
Task: "Implement RegistersPane in crates/rde-tui/src/panes/registers.rs"
Task: "Implement StackPane in crates/rde-tui/src/panes/stack.rs"
Task: "Implement BreakpointsPane in crates/rde-tui/src/panes/breakpoints.rs"

# Then integrate layout and focus management:
Task: "Implement layout rendering in crates/rde-tui/src/app.rs"
Task: "Implement focus management in crates/rde-tui/src/app.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 (four panes + layout + `--tui` flag)
4. **STOP and VALIDATE**: Launch TUI, attach to debuggee, verify all panes render correctly
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Polish (status bar, help, keybindings, docs)
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: US1 — SourceAsmPane + RegistersPane
   - Developer B: US1 — StackPane + BreakpointsPane + layout integration
   - Developer C: US2 — ReplPane (can proceed in parallel once app.rs layout exists)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
