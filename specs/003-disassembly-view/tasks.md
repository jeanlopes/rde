# Tasks: Disassembly View

**Input**: Design documents from `/specs/003-disassembly-view/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included as this feature benefits from integration testing against a real debuggee process.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add Capstone dependency and verify workspace compilation

- [X] T001 Add `capstone = "0.12"` dependency to `crates/rde-core/Cargo.toml` with features appropriate for x86-64 disassembly
- [X] T002 Verify workspace compiles with `cargo check --workspace` after adding Capstone

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types, events, and cross-crate APIs that MUST exist before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 [P] Add `EngineCommand::Disassemble` variant to `crates/rde-core/src/events.rs` with fields `{ address: Option<u64>, thread_id: Option<u32>, count: Option<usize> }`
- [X] T004 [P] Add `DisassemblyLine`, `DisassemblyView`, `DisassemblyConfig` structs to `crates/rde-core/src/disasm.rs` per data-model.md
- [X] T005 Create `crates/rde-core/src/disasm.rs` module skeleton with Capstone initialization (`Capstone::new().x86().mode(Mode64).syntax(Intel).build()`)
- [X] T006 Add `pub mod disasm;` and re-export types in `crates/rde-core/src/lib.rs`
- [X] T007 [P] Add `list_active() -> HashSet<u64>` method to breakpoint manager in `crates/rde-breakpoint/src/lib.rs` (or verify it exists)
- [X] T008 [P] Add `resolve_by_name(name: &str) -> Result<Vec<u64>, DebugError>` to `SymbolEngine` trait in `crates/rde-symbols/src/lib.rs` (or verify it exists)
- [X] T009 [P] Verify `rde-win32` exposes a safe `read_memory(handle: &ProcessHandle, address: u64, size: usize) -> Result<Vec<u8>, DebugError>` wrapper (or create it in `crates/rde-win32/src/process.rs`)
- [X] T010 [P] Verify `rde-win32` exposes `get_thread_context(handle: RawHandle) -> Result<CONTEXT, DebugError>` (or equivalent) for RIP extraction

**Checkpoint**: Foundation ready — workspace compiles, types exist, cross-crate APIs available

---

## Phase 3: User Story 1 — Disassemble at RIP on Breakpoint Hit (Priority: P1) 🎯 MVP

**Goal**: Comando `disassemble` sem argumentos mostra ~10 instruções ao redor do RIP do thread selecionado, com instrução atual marcada por `=>`

**Independent Test**: Executar debuggee `hello_debuggee`, setar breakpoint, dar `continue`, e quando parar executar `disassemble`. Verificar que output contém instruções x86-64 válidas e `=>` na linha do RIP.

### Tests for User Story 1

- [X] T011 [P] [US1] Write integration test in `tests/integration_tests.rs`: launch debuggee, hit breakpoint, send `EngineCommand::Disassemble { address: None, .. }`, assert output contains `=>` and valid x86-64 mnemonics
- [X] T012 [P] [US1] Add golden path snapshot `test_data/golden_paths/003_disassembly_rip.txt` with expected disassembly output format

### Implementation for User Story 1

- [X] T013 [US1] Implement `Disassembler::disassemble_at_rip(&self, session: &Session, count: usize) -> Result<DisassemblyView, DebugError>` in `crates/rde-core/src/disasm.rs`
  - Extract RIP from selected thread (or thread 0 fallback) via `GetThreadContext`
  - Calculate start address: `RIP.saturating_sub(offset)` (offset = ~5 instruções × 8 bytes heurística)
  - Read memory via `ReadProcessMemory` wrapper (limit: `min(64 × count, 4096)`)
  - Disassemble with Capstone
  - Build `DisassemblyView` with `lines` and `rip` set
- [X] T014 [US1] Implement `DisassemblyView::format() -> String` in `crates/rde-core/src/disasm.rs` producing aligned columns: address | bytes (≤8, `..` if truncated) | mnemonic | operands | markers (`=>` for RIP)
- [X] T015 [US1] Add `EngineCommand::Disassemble` handler in `crates/rde-core/src/engine.rs` that calls `Disassembler::disassemble_at_rip` and emits `EngineEvent::Output(formatted_text)`
- [X] T016 [US1] Add `disas` and `disassemble` command parsing (no arguments) in `crates/rde-repl/src/parser.rs` returning `EngineCommand::Disassemble { address: None, thread_id: None, count: None }`
- [X] T017 [US1] Handle `EngineEvent::Output` rendering in `crates/rde-repl/src/lib.rs` (display raw text followed by newline)
- [X] T018 [US1] Add `tracing::info!` / `tracing::debug!` instrumentation in the disassembly path: command received, memory read (address + size), Capstone result (instruction count), formatting complete

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 — Disassemble at Arbitrary Address (Priority: P2)

**Goal**: Comando `disassemble <address>` e `disassemble <symbol>` funciona, resolvendo símbolos via `rde-symbols`; símbolos ambíguos retornam erro listando matches

**Independent Test**: Carregar debuggee, executar `modules` para obter base address, executar `disassemble <base>` e verificar entry point. Executar `disassemble main` e verificar resolução (ou erro ambíguo se aplicável).

### Tests for User Story 2

- [X] T019 [P] [US2] Write unit test in `crates/rde-repl/src/parser.rs`: verify `disas 0x140001000` parses to `EngineCommand::Disassemble { address: Some(0x140001000), .. }`
- [X] T020 [P] [US2] Write unit test in `crates/rde-repl/src/parser.rs`: verify `disas main` parses to `EngineCommand::Disassemble { address: None, .. }` with symbol flag or equivalent

### Implementation for User Story 2

- [X] T021 [US2] Extend parser in `crates/rde-repl/src/parser.rs` to accept:
  - `disas <hex>` / `disassemble <hex>` — parse `0x...` or raw hex string to `u64`
  - `disas <symbol>` / `disassemble <symbol>` — pass symbol name to engine
- [X] T022 [US2] Extend `EngineCommand::Disassemble` (or add new variant) to carry `symbol: Option<String>` if não for possível distinguir hex de símbolo no parser; alternativamente, resolver no engine tentando parse hex primeiro, fallback para símbolo
- [X] T023 [US2] Implement `Disassembler::disassemble_at_address(address: u64, session: &Session, count: usize) -> Result<DisassemblyView, DebugError>` in `crates/rde-core/src/disasm.rs` (reusa lógica de leitura/disassembly de T013, sem centragem em RIP)
- [X] T024 [US2] Add symbol resolution path in `crates/rde-core/src/engine.rs`: if input looks like symbol (not valid hex), call `SymbolEngine::resolve_by_name(name)`; if result is empty → `Symbol not found`; if result.len() > 1 → `Ambiguous symbol 'xxx': found at 0x..., 0x...`; if exactly 1 → disassemble at that address
- [X] T025 [US2] Add error formatting for disassembly failures in `crates/rde-core/src/engine.rs`: "Cannot read memory at 0x...: [reason]", "Symbol 'xxx' not found", "Ambiguous symbol 'xxx': ..."

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 — Breakpoint Highlighting in Disassembly (Priority: P2)

**Goal**: Instruções com breakpoints ativos exibem `b+`; instruções shadowed por INT3 mostram `int3 ; was: <original>`

**Independent Test**: Setar breakpoint em endereço conhecido, executar `disassemble` no range cobrindo esse endereço, verificar `b+` e comentário `; was:`.

### Tests for User Story 3

- [X] T026 [P] [US3] Write integration test in `tests/integration_tests.rs`: set breakpoint, hit it, run `disassemble`, assert output contains `b+` at breakpoint address and `=>` at RIP (both markers can coexist)
- [X] T027 [P] [US3] Write unit test in `crates/rde-core/src/disasm.rs`: given mock DisassemblyView with `has_breakpoint=true`, verify formatted output contains `b+` prefix

### Implementation for User Story 3

- [X] T028 [US3] Extend `DisassemblyLine` with `original_bytes: Option<Vec<u8>>` field in `crates/rde-core/src/disasm.rs`
- [X] T029 [US3] During `Disassembler::disassemble_*` execution, query active breakpoints via `BreakpointManager::list_active()` → `HashSet<u64>`; for each decoded instruction, set `has_breakpoint` if address ∈ set
- [X] T030 [US3] For breakpoints de software (INT3), retrieve byte original do breakpoint manager (ou do `Session` se o manager armazena bytes originais); decode original bytes com Capstone para obter mnemonic/operands; store in `original_bytes` + set `has_breakpoint = true`
- [X] T031 [US3] Update `DisassemblyView::format()` to:
  - Prefix line with `b+` (e `=>` se for RIP também) quando `has_breakpoint == true`
  - Append `; was: <mnemonic> <operands>` quando `original_bytes` is Some
- [X] T032 [US3] Ensure marker precedence/format: `=> b+` quando RIP == breakpoint address; `b+` sozinho quando breakpoint != RIP; `=>` sozinho quando RIP sem breakpoint

**Checkpoint**: All breakpoint highlighting should now work; US1 and US2 unaffected

---

## Phase 6: User Story 4 — Sincronia Automática com RIP (Priority: P3)

**Goal**: Modo `set auto-disassemble on|off` e `set disassembly-count N`; auto-exibe disassembly após paradas

**Independent Test**: Habilitar `auto-disassemble on`, dar `step`, verificar que disassembly aparece automaticamente antes do prompt; desabilitar, dar `step`, verificar que NÃO aparece.

### Tests for User Story 4

- [X] T033 [P] [US4] Write unit test in `crates/rde-repl/src/parser.rs`: verify `set auto-disassemble on` / `off` parses correctly
- [X] T034 [P] [US4] Write unit test in `crates/rde-repl/src/parser.rs`: verify `set disassembly-count 20` parses correctly
- [X] T035 [P] [US4] Write integration test in `tests/integration_tests.rs`: enable auto-disassemble, step, assert output contains disassembly lines automatically

### Implementation for User Story 4

- [X] T036 [US4] Add `DisassemblyConfig` to REPL state in `crates/rde-repl/src/lib.rs` (or session state if persistido cross-command): `count: usize = 10`, `auto_show: bool = false`
- [X] T037 [US4] Add `set auto-disassemble on|off` parsing in `crates/rde-repl/src/parser.rs`
- [X] T038 [US4] Add `set disassembly-count <N>` parsing in `crates/rde-repl/src/parser.rs` with validação (N ≥ 1, N ≤ 100)
- [X] T039 [US4] Implement config update handlers in `crates/rde-repl/src/lib.rs` (local REPL state, não precisa ir ao engine)
- [X] T040 [US4] Modify REPL event loop in `crates/rde-repl/src/lib.rs`: após receber `EngineEvent::BreakpointHit`, `SingleStep`, ou `Exception`, se `auto_show == true`, enviar automaticamente `EngineCommand::Disassemble { address: None, thread_id: Some(event_thread_id), count: Some(config.count) }` antes de exibir o prompt
- [X] T041 [US4] Ensure auto-disassemble respeita estado do target: se target está running (não deveria acontecer pois eventos só chegam quando parado), suprimir silenciosamente

**Checkpoint**: Auto-disassemble fully functional; all previous stories still work

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Contracts, tracing validation, golden paths, and documentation

- [X] T042 [P] Verify `ReadProcessMemory` wrapper in `crates/rde-win32/src/process.rs` enforces max 4096 bytes invariant; add `debug_assert!(size <= 4096)` or equivalent
- [X] T043 [P] Add/update contract doc `docs/contracts/read-process-memory.md` with final implementation details (reference actual wrapper function signature)
- [X] T044 Validate tracing coverage: run `cargo test` with `RUST_LOG=trace` and verify disassembly path emits logs at: command received, memory read (address + bytes_read), Capstone call (instruction count), formatting, errors
- [X] T045 [P] Update `docs/quickstart.md` with final command syntax and examples from quickstart.md in feature dir
- [X] T046 Run `cargo test --workspace` and fix any compiler warnings or errors introduced by the feature
- [X] T047 [P] Update `test_data/golden_paths/003_disassembly_rip.txt` with actual captured output from a successful integration test run
- [X] T048 Code review: verify no `unsafe` outside `rde-win32`; verify all new public APIs have `///` doc comments

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3–6)**: All depend on Foundational phase completion
  - US1 (P1) → US2 (P2) → US3 (P2) → US4 (P3) recommended sequential order
  - US2 and US3 can be parallel if team capacity allows (both depend only on US1 foundation)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — No dependencies on other stories; **MVP**
- **User Story 2 (P2)**: Can start after US1 complete — needs parser extension and address resolution
- **User Story 3 (P2)**: Can start after US1 complete — needs breakpoint query integration
- **User Story 4 (P3)**: Can start after US1 complete — needs disassembly command already working

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Types/entities before services/formatting
- Parser before engine handler
- Core implementation before integration

### Parallel Opportunities

- T003–T010 (Foundational) can run in parallel (different crates/files)
- T011–T012 (US1 tests) can run in parallel
- T019–T020 (US2 tests) can run in parallel
- T026–T027 (US3 tests) can run in parallel
- T033–T035 (US4 tests) can run in parallel
- T042–T043, T047 (Polish) can run in parallel
- US2 and US3 implementation can be worked on in parallel after US1 is done

---

## Parallel Example: User Story 1

```bash
# Launch tests together:
Task: "T011 Write integration test for disassemble at RIP"
Task: "T012 Add golden path snapshot"

# Launch foundational models together:
Task: "T013 Implement Disassembler::disassemble_at_rip"
Task: "T014 Implement DisassemblyView::format()"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL)
3. Complete Phase 3: User Story 1 (disassemble at RIP, basic formatting)
4. **STOP and VALIDATE**: Test with real debuggee, verify `=>` marker and valid x86-64 output
5. Demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Demo (MVP!)
3. Add User Story 2 → Test independently (address + symbol resolution)
4. Add User Story 3 → Test independently (breakpoint highlighting)
5. Add User Story 4 → Test independently (auto-disassemble)
6. Polish phase → contracts, tracing, golden paths

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational done:
   - Developer A: US1 + US4 (REPL-centric)
   - Developer B: US2 + US3 (engine-centric)
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
- **Total tasks**: 48
- **Tasks per story**: US1=8, US2=6, US3=5, US4=6
- **Foundational tasks**: 8
- **Polish tasks**: 7
