# Feature Specification: Features Left Behind

**Feature Branch**: `007-features-left-behind`

**Created**: 2026-06-01

**Status**: Draft

**Input**: Derived from 90 `#[ignore]` test cases in `tests/big_test_plan.rs` that could not run because their underlying features were not yet implemented.

---

## User Scenarios & Testing *(mandatory)*

<!--
  Each user story maps to one or more groups of ignored TCs.
  TCs are grouped by shared blocking cause; similar ones are merged into a single story.
  Priority reflects debugger core-value ordering: execution control first, then inspection, then tooling.
-->

### User Story 1 — Step Over Execution (Priority: P1)

As a developer debugging a program, I want to step over function calls so that I can advance the program by one source line without descending into called functions.

**Why this priority**: Step over is one of the three fundamental debugger execution primitives. Without it, debugging loops and call sites requires many manual `continue`/`break` cycles. Unblocks 14 tests.

**Independent Test**: Set a breakpoint on `insert`, hit it, issue `next` once, and verify the debugger stops at the next line of `insert` rather than entering a callee.

**Acceptance Scenarios**:

1. **Given** a breakpoint is hit on `insert`, **When** the user sends `next`, **Then** execution advances one source line within `insert` without descending into recursive calls.
2. **Given** stepping over a call that does not reach another breakpoint, **When** `next` is issued, **Then** the REPL prompt returns and no spurious breakpoint event fires.
3. **Given** multiple `next` commands in sequence on `inorder_traversal`, **When** each is issued, **Then** each advances exactly one line.

**Covered TCs**: TC-079 to TC-089, TC-103, TC-117, TC-260

---

### User Story 2 — Step Out Execution (Priority: P1)

As a developer stopped deep inside a recursive function, I want to step out so that I can return to the caller without stepping through every remaining line.

**Why this priority**: Completes the standard step triad (into / over / out). Without it, users are stuck inside recursive functions until they continue past all breakpoints. Unblocks 12 tests.

**Independent Test**: Set a breakpoint on `insert`, hit it, issue `finish`, and verify the REPL stops in the caller frame.

**Acceptance Scenarios**:

1. **Given** execution is stopped inside `insert`, **When** the user sends `finish`, **Then** execution runs to the end of `insert` and the debugger stops in the caller.
2. **Given** 5 levels of recursion have been stepped into, **When** `finish` is issued 5 times, **Then** each call unwinds one frame.
3. **Given** a breakpoint is set at the return site, **When** `finish` is issued, **Then** the breakpoint at the return site fires normally.

**Covered TCs**: TC-090 to TC-097, TC-104, TC-107, TC-118, TC-159

---

### User Story 3 — Cargo Debug Integration (Priority: P1)

As a developer in a Cargo workspace, I want to launch rde-cli with `cargo debug` so that it automatically builds and debugs the project without manually locating the binary.

**Why this priority**: Cargo is the standard Rust build system; most users will reach for `cargo debug` rather than pointing at a binary. Unblocks 7 tests.

**Independent Test**: Run `rde-cli cargo debug` in a Cargo workspace — verify the binary is built (if stale) and the debug session opens at the REPL prompt.

**Acceptance Scenarios**:

1. **Given** a Cargo workspace, **When** `rde-cli cargo debug` is run, **Then** the binary is built in debug profile and a session opens.
2. **Given** `--release` flag is passed, **When** `rde-cli cargo debug --release` is run, **Then** the binary is built in release profile.
3. **Given** `--features avl,tracing` flags are passed, **When** launched, **Then** the binary is built with those features enabled.
4. **Given** `--package rust_app_example --bin rust_app_example` flags are passed, **When** launched, **Then** that specific crate/binary is selected.
5. **Given** the binary is stale (source changed since last build), **When** launched, **Then** the binary is rebuilt before the session opens.
6. **Given** the build fails (syntax error in source), **When** launched, **Then** a clear build-failure error is displayed and no debug session is started.

**Covered TCs**: TC-003, TC-004, TC-005, TC-006, TC-011, TC-012, TC-263

---

### User Story 4 — Running State Command Rejection (Priority: P2)

As a developer, when the target process is actively running (not paused at a breakpoint), I want any REPL command that requires the process to be paused to return a clear error rather than corrupting state.

**Why this priority**: Without this, sending `step` or `continue` while the process runs produces undefined behavior. Unblocks 2 tests.

**Independent Test**: Send `continue` to a running session, then immediately send `step`; verify a graceful error message is returned.

**Acceptance Scenarios**:

1. **Given** the process is running (after `continue` with no breakpoints), **When** the user sends `step`, **Then** the REPL responds with a "process is running" error and does not crash.
2. **Given** the process is running, **When** the user sends `continue` again, **Then** the REPL responds gracefully (error or no-op) without hanging.

**Covered TCs**: TC-274, TC-275

---

### User Story 5 — Dynamic Breakpoint Management During Execution (Priority: P2)

As a developer, I want to add or remove breakpoints while the target process is running between pauses so that I do not need to restart the session to adjust breakpoints.

**Why this priority**: Practical debugging sessions require adjusting breakpoints mid-run. Unblocks 2 tests.

**Independent Test**: Send `continue` (no breakpoints set), then while the process runs send `break insert`; verify the breakpoint fires on the next call to `insert`.

**Acceptance Scenarios**:

1. **Given** the process is running, **When** the user sends `break insert`, **Then** the breakpoint is registered and fires the next time `insert` is called.
2. **Given** a breakpoint is set and the process is running, **When** the user sends `delbreak 1`, **Then** the breakpoint is removed and does not fire again.

**Covered TCs**: TC-036, TC-037

---

### User Story 6 — Process Attach (Priority: P2)

As a developer, I want to attach rde-cli to an already-running process by PID so that I can debug processes I did not launch.

**Why this priority**: Attach is essential for debugging long-running services or reproducing hard-to-launch bugs. Unblocks 1 test.

**Independent Test**: Start `rust_app_example` independently, note its PID, then run `rde-cli <binary>` and issue `attach <PID>`; verify the session attaches.

**Acceptance Scenarios**:

1. **Given** a process is running with PID `P`, **When** the user issues `attach P`, **Then** the debugger attaches and the REPL becomes active.
2. **Given** an invalid or non-existent PID, **When** `attach` is issued, **Then** a clear error is returned without crashing.

**Covered TCs**: TC-008

---

### User Story 7 — Extended Symbol Resolution (Priority: P2)

As a developer, I want to set breakpoints on standard library symbols (`std::io::_print`, `drop_in_place`), private helper functions, and derived trait methods so that I can debug all code paths, not only public user-defined functions.

**Why this priority**: Real debugging sessions frequently require stopping in stdlib or derived code. Unblocks 4 tests.

**Independent Test**: Issue `break std::io::_print` and verify that hitting `continue` stops at a `println!` call site.

**Acceptance Scenarios**:

1. **Given** a session, **When** `break std::io::_print` is issued, **Then** a breakpoint is confirmed and fires when `println!` executes.
2. **Given** a session, **When** `break drop_in_place` is issued, **Then** a breakpoint fires when a `Box<TreeNode>` is dropped.
3. **Given** a private helper `rebalance` function exists in the debuggee, **When** `break rebalance` is issued, **Then** the breakpoint is created and fires.
4. **Given** `PartialEq` is derived on `TreeNode`, **When** `break eq` is issued, **Then** the breakpoint fires when equality is tested.

**Covered TCs**: TC-046, TC-047, TC-048, TC-055

---

### User Story 8 — AVL Rotation & Recursive Delete in Debuggee (Priority: P2)

As a developer verifying AVL tree behavior, I want the `rust_app_example` demo scenarios to actually trigger `rotate_left`, `rotate_right`, and recursive delete with `find_min` so that those code paths can be debugged.

**Why this priority**: Several tests are blocked not by missing rde-cli features but by the debuggee not exercising those code paths. The fix is in the debuggee, not the engine. Unblocks 5 tests.

**Independent Test**: Run `--demo insert-sequence` with sufficient insertions to require an AVL rotation; verify `break rotate_left` fires.

**Acceptance Scenarios**:

1. **Given** `--demo insert-sequence`, **When** enough values are inserted to unbalance the AVL tree, **Then** `rotate_left` is called and a breakpoint on it fires.
2. **Given** `--demo insert-sequence`, **When** a right-heavy imbalance occurs, **Then** `rotate_right` is called and a breakpoint fires.
3. **Given** `--demo delete-rebalance`, **When** a node with two children is deleted, **Then** `find_min` is called to find the inorder successor.
4. **Given** `--demo delete-rebalance`, **When** a node is deleted, **Then** the deletion recurses and a breakpoint on `delete` fires more than once.

**Covered TCs**: TC-068, TC-069, TC-070, TC-071, TC-157

---

### User Story 9 — Pretty Print Configuration Commands (Priority: P2)

As a developer, I want to configure pretty-printing behavior at runtime via `set print-limit N`, `set print-depth N`, and `set pretty-print on|off` REPL commands so that I can control output verbosity for large or deeply nested data structures.

**Why this priority**: `set auto-disassemble` and `set disassembly-count` already work; the same pattern needs to extend to print controls. Unblocks 5 tests.

**Independent Test**: Issue `set print-limit 5`, then `print result` on a Vec with more than 5 elements; verify only 5 elements are shown.

**Acceptance Scenarios**:

1. **Given** `set print-limit 5` is issued, **When** a Vec with 150 elements is printed, **Then** only 5 elements are shown and a truncation indicator appears.
2. **Given** `set print-depth 2` is issued, **When** a nested `TreeNode` is printed, **Then** only 2 levels of nesting are expanded.
3. **Given** `set pretty-print off` is issued, **When** a value is printed, **Then** the output is raw/compact rather than structured.
4. **Given** `print --depth 10 root` is issued on a deep tree, **When** the print budget is exceeded, **Then** output is truncated with a clear indicator.
5. **Given** a large Vec from the stress-test traversal, **When** `print result` is issued, **Then** output is produced (possibly truncated) without crashing.

**Covered TCs**: TC-165, TC-174, TC-175, TC-176, TC-183

---

### User Story 10 — Custom Struct & Enum Types in Debuggee (Priority: P3)

As a developer, I want `rust_app_example` to expose a `TreeStats` struct and a `TreeError` enum so that I can verify pretty-printing of custom named types.

**Why this priority**: Debuggee enhancement; tests are blocked by missing types, not missing engine features. Unblocks 2 tests.

**Independent Test**: Break inside `insert`, issue `print stats` (a `TreeStats` instance), and verify the struct fields are displayed.

**Acceptance Scenarios**:

1. **Given** a `TreeStats { insertions: usize, rotations: usize }` struct is visible at a breakpoint, **When** `print stats` is issued, **Then** the field names and values are displayed.
2. **Given** a `TreeError::DuplicateValue` enum variant is in scope, **When** `print err` is issued, **Then** the variant name and payload are displayed.

**Covered TCs**: TC-177, TC-178

---

### User Story 11 — TUI Mode (Priority: P2)

As a developer who prefers a full-screen interface, I want to launch rde-cli with `--tui` so that I get a terminal UI with panels for source, registers, and output instead of a plain REPL.

**Why this priority**: TUI is a differentiating user experience feature. Unblocks 2 tests.

**Independent Test**: Launch `rde-cli --tui <binary>`, verify the ratatui interface appears and `q` quits cleanly.

**Acceptance Scenarios**:

1. **Given** `rde-cli --tui <binary>` is run, **When** the process starts, **Then** a full-screen ratatui layout is displayed with at least a status pane.
2. **Given** the TUI is active, **When** the user presses `q`, **Then** the UI exits cleanly and the terminal is restored.

**Covered TCs**: TC-007, TC-269

---

### User Story 12 — Multi-Thread Debuggee (Priority: P2)

As a developer debugging concurrent code, I want `rust_app_example` to provide a multi-threaded variant so that rde-cli's thread-related commands can be exercised.

**Why this priority**: Many thread-related tests are blocked because the current debuggee is single-threaded. Unblocks 5 tests.

**Independent Test**: Launch the threaded debuggee; run `threads` and verify at least 2 thread entries appear.

**Acceptance Scenarios**:

1. **Given** the threaded debuggee launches, **When** `threads` is issued, **Then** at least 2 threads are listed (main + worker).
2. **Given** the threaded debuggee runs, **When** a worker thread starts, **Then** a "thread created" event appears in the session output.
3. **Given** the threaded debuggee exits, **When** the process ends, **Then** thread exit events appear before the process exit event.

**Covered TCs**: TC-212, TC-221, TC-229, TC-230, TC-267

---

### User Story 13 — Thread Selection Command (Priority: P2)

As a developer, I want to switch the active debug context to a specific thread by issuing `thread <id>` so that `bt`, `regs`, and `print` commands operate on that thread's context.

**Why this priority**: Without thread switching, all inspection commands apply only to the first thread. Unblocks 6 tests.

**Independent Test**: List threads, pick one ID, issue `thread <ID>`, then `bt`; verify the backtrace reflects that thread's stack.

**Acceptance Scenarios**:

1. **Given** at least one thread is listed by `threads`, **When** `thread <valid_id>` is issued, **Then** the current context switches to that thread and subsequent `regs`/`bt` commands reflect its state.
2. **Given** the `threads` output, **When** the selected thread is displayed, **Then** a `*` marker appears next to the active thread.
3. **Given** `thread <invalid_id>` is issued, **When** the id does not match any thread, **Then** a clear error is returned.

**Covered TCs**: TC-213, TC-214, TC-222, TC-223, TC-224, TC-228

---

### User Story 14 — Tokio Async Task Inspection (Priority: P2)

As a developer building async Rust applications, I want the `tasks` command to list Tokio runtime tasks when the debuggee uses Tokio so that I can inspect async task state.

**Why this priority**: rde-tokio crate already exists; this connects it to a real debuggee. Unblocks 1 test.

**Independent Test**: Create a minimal Tokio debuggee, attach the debugger, issue `tasks`, and verify task IDs and states appear.

**Acceptance Scenarios**:

1. **Given** a Tokio-based debuggee is paused, **When** `tasks` is issued, **Then** a table of task IDs, states, and function names is displayed.

**Covered TCs**: TC-219

---

### User Story 15 — Module Load Event Tracking (Priority: P3)

As a developer, I want `modules` to reflect DLLs that have been dynamically loaded after launch so that I can see the full module list at any point during execution.

**Why this priority**: Nice-to-have for completeness; Win32 `LOAD_DLL_DEBUG_EVENT` already fires in the debug loop. Unblocks 1 test.

**Independent Test**: Launch a debuggee that loads a DLL lazily; issue `modules` after the load event; verify the DLL appears.

**Acceptance Scenarios**:

1. **Given** a DLL is loaded after process start, **When** `modules` is issued after the load event fires, **Then** the newly loaded DLL appears in the module list.

**Covered TCs**: TC-217

---

### User Story 16 — Heap Pointer Inspection (Priority: P3)

As a developer, I want to use `print root` to obtain the address of a heap-allocated `TreeNode` and then examine that address with `x` so that I can inspect the raw memory layout of Rust structs.

**Why this priority**: Advanced memory inspection workflow. Blocked by address extraction from `print` output. Unblocks 4 tests.

**Independent Test**: Break inside `insert`, `print root`, parse the printed address, issue `x <address> 24`, and verify the bytes are readable.

**Acceptance Scenarios**:

1. **Given** `print root` displays a pointer address, **When** that address is used in `x <addr> 24`, **Then** the memory contents at that address are displayed.
2. **Given** a `Box<TreeNode>` is allocated via `Box::new`, **When** the pointer is examined after the allocation, **Then** the expected struct fields are visible at that address.
3. **Given** a node has been dropped, **When** its former address is examined, **Then** the memory is still readable (implementation-defined content, no crash).

**Covered TCs**: TC-247, TC-248, TC-249, TC-250

---

### User Story 17 — Step Into Compiler & Runtime Internals (Priority: P3)

As an advanced developer, I want `step` to descend into compiler-generated code such as `Box::new` allocations, `Drop` implementations, standard library functions, and inlined functions so that I can inspect the full execution path.

**Why this priority**: Edge case; step into std requires symbol files for stdlib. Unblocks 4 tests.

**Independent Test**: Break on `insert`, step until entering a call to `Box::new`, verify the backtrace shows allocator frames.

**Acceptance Scenarios**:

1. **Given** the debugger is stopped just before a `Box::new` call, **When** `step` is issued, **Then** execution enters the allocator code (or the first user-visible instruction inside it).
2. **Given** a `Drop` implementation is reachable, **When** `step` reaches the drop, **Then** execution enters the drop body.
3. **Given** a stdlib function like `Vec::push` is called, **When** `step` is issued, **Then** execution descends into it.

**Covered TCs**: TC-077, TC-078, TC-098, TC-099

---

### User Story 18 — Launch Edge Cases (Priority: P3)

As a developer, I want rde-cli to behave predictably when given unusual inputs so that errors are communicated clearly.

**Why this priority**: Robustness polish. Unblocks 2 tests.

**Independent Test**: Launch rde-cli with `Cargo.toml` as the binary path; verify a clear error is shown.

**Acceptance Scenarios**:

1. **Given** a non-executable file is passed as the binary, **When** rde-cli is launched, **Then** a clear error message is displayed and no debug session is started.
2. **Given** an active session, **When** a second `launch <path>` command is issued, **Then** the behavior is defined: either the current session is replaced or a clear error is returned.

**Covered TCs**: TC-014, TC-015

---

### User Story 19 — Script / Batch Input Mode (Priority: P3)

As a developer running automated tests, I want to pipe a sequence of REPL commands to rde-cli via stdin so that I can run non-interactive debug sessions from scripts.

**Why this priority**: Enables CI scripting without the test harness. Unblocks 1 test.

**Independent Test**: Echo `break main\ncontinue\nquit\n` and pipe it to `rde-cli <binary>`; verify the session processes all commands and exits.

**Acceptance Scenarios**:

1. **Given** a newline-delimited list of REPL commands is piped to stdin, **When** rde-cli starts in non-interactive mode, **Then** each command is executed in order and the process exits after the last command.

**Covered TCs**: TC-261

---

### User Story 20 — Golden Path Snapshot Testing (Priority: P3)

As a maintainer, I want a golden-path test that captures normalized rde-cli output and compares it against a stored snapshot so that regressions in output format are immediately detected.

**Why this priority**: Snapshot tests are a long-term quality gate. Unblocks 1 test.

**Independent Test**: Run the golden path scenario, capture normalized output, compare against the stored snapshot file; verify no diff.

**Acceptance Scenarios**:

1. **Given** a stored golden snapshot exists, **When** the full golden-path scenario is executed, **Then** the normalized output matches the snapshot with no unexpected differences.

**Covered TCs**: TC-262

---

### User Story 21 — Panic Debuggee Scenario (Priority: P3)

As a developer, I want `rust_app_example` to support `--demo panic` so that I can verify rde-cli handles a panicking process gracefully.

**Why this priority**: Adds a missing demo mode to the debuggee. Unblocks 1 test.

**Independent Test**: Run `rde-cli <binary> --demo panic`, `continue`; verify the session reports the panic/exception event and exits cleanly.

**Acceptance Scenarios**:

1. **Given** `--demo panic` launches a process that calls `panic!("test panic")`, **When** `continue` is issued, **Then** rde-cli reports a panic/exception event and the session terminates gracefully.

**Covered TCs**: TC-266

---

### User Story 22 — Privilege-Aware Attach Error (Priority: P3)

As a developer on Windows, I want `attach 4` (the System process) to return a meaningful "access denied" error rather than a generic crash so that the permissions boundary is clearly communicated.

**Why this priority**: Error quality for a known Windows-specific edge case. Unblocks 1 test.

**Independent Test**: Issue `attach 4`; verify the response contains "acesso negado" or "access denied" without crashing the CLI.

**Acceptance Scenarios**:

1. **Given** a process protected by Windows security (PID 4), **When** `attach 4` is issued, **Then** an "access denied" error is displayed and the session remains alive.

**Covered TCs**: TC-279

---

### User Story 23 — Performance & Stability Test Infrastructure (Priority: P3)

As a maintainer, I want tests that measure timing of key operations (first breakpoint hit with 50 breakpoints set, 1000-node stress-test run time) and memory stability over long sessions so that performance regressions are detectable.

**Why this priority**: Quality gate for scale. Unblocks 5 tests.

**Independent Test**: Set 17 breakpoints, start a session, measure time-to-first-hit; assert it is under 2 seconds.

**Acceptance Scenarios**:

1. **Given** 17+ breakpoints are set, **When** `continue` is issued, **Then** the first breakpoint fires in under 2 seconds.
2. **Given** the stress-test demo (100 insertions) runs to completion, **When** no breakpoints are set, **Then** the session exits in under 5 seconds.
3. **Given** a 50-iteration loop of `continue` → hit → `vars` → `regs` → `bt`, **When** the loop completes, **Then** no memory leaks or panics occur in rde-cli.

**Covered TCs**: TC-119, TC-257, TC-288, TC-289, TC-290

---

### User Story 24 — Full Regression Meta-Test (Priority: P3)

As a CI maintainer, I want a single meta-test that exercises the full golden path in one test run so that a green run proves all core features work together.

**Why this priority**: CI quality gate. Unblocks 1 test.

**Independent Test**: Run the meta-test; assert `Processo iniciado` and `Processo encerrado` appear in normalized output.

**Acceptance Scenarios**:

1. **Given** the full golden-path scenario runs, **When** the meta-test executes, **Then** both process-start and process-exit events appear in normalized output.

**Covered TCs**: TC-270

---

### Edge Cases

- What happens when `finish` is issued at the outermost frame (main)?
- How does `set print-limit 0` behave — unlimited or zero items?
- What if `attach` targets a process on a different user account?
- What if `cargo debug` is run outside a Cargo workspace?
- What if `step` is issued repeatedly and the instruction pointer leaves debuggee address space?
- How does thread selection interact with breakpoints — does `bt` after `thread <id>` switch show the correct frame even if that thread is not the one that hit the breakpoint?

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The REPL MUST support a `next` command that advances execution by one source line without descending into function calls.
- **FR-002**: The REPL MUST support a `finish` command that runs the current function to completion and returns to the caller.
- **FR-003**: `rde-cli` MUST accept a `cargo debug [options]` subcommand that builds and launches a Cargo project under the debugger.
- **FR-004**: `cargo debug` MUST support `--release`, `--features <list>`, `--package <name>`, and `--bin <name>` flags.
- **FR-005**: `cargo debug` MUST detect a stale binary (source modified after last build) and rebuild before launching.
- **FR-006**: The REPL MUST respond with a clear error when a control command (`step`, `continue`) is issued while the process is already running.
- **FR-007**: The REPL MUST allow adding and removing breakpoints while the target process is running.
- **FR-008**: `rde-cli` MUST support `attach <pid>` to attach to an already-running process.
- **FR-009**: Breakpoint resolution MUST support standard library symbols, compiler-generated drop glue, private functions, and derived trait methods when debug info is available.
- **FR-010**: `rust_app_example` MUST be enhanced so that `--demo insert-sequence` triggers AVL rotations (`rotate_left`/`rotate_right`) and `--demo delete-rebalance` exercises recursive deletion via `find_min`.
- **FR-011**: The REPL MUST support `set print-limit N`, `set print-depth N`, and `set pretty-print on|off` commands.
- **FR-012**: `rust_app_example` MUST expose a `TreeStats` struct and `TreeError` enum in a breakpoint-reachable scope.
- **FR-013**: `rde-cli` MUST support a `--tui` flag that activates a ratatui full-screen interface.
- **FR-014**: `rust_app_example` MUST provide a multi-threaded demo variant that spawns at least one worker thread.
- **FR-015**: The REPL MUST support `thread <id>` to switch the active inspection context to a specified thread.
- **FR-016**: `threads` output MUST mark the currently selected thread with a `*` indicator.
- **FR-017**: A Tokio-based debuggee example MUST exist for testing the `tasks` command.
- **FR-018**: The `modules` command MUST reflect DLLs loaded dynamically after process start.
- **FR-019**: `print <var>` output MUST include a pointer address for heap-allocated types, usable as input to the `x` command.
- **FR-020**: `rde-cli` MUST process a newline-delimited command script piped via stdin without interactive prompts.
- **FR-021**: A golden-path snapshot file MUST exist and a test MUST compare normalized session output against it.
- **FR-022**: `rust_app_example` MUST support `--demo panic` to exercise panic handling.
- **FR-023**: `attach 4` on Windows MUST return an "access denied" error without crashing.
- **FR-024**: Once a feature is implemented and its corresponding tests pass locally, all `#[ignore]` tags for that feature MUST be removed from `tests/big_test_plan.rs`. No test may remain ignored unless it is explicitly deprecated (in which case it must be deleted, not re-ignored).

### Key Entities

- **`next` command**: Step-over execution primitive; advances one source line in the current frame.
- **`finish` command**: Step-out execution primitive; runs to end of current frame.
- **`cargo debug` subcommand**: Wrapper that invokes `cargo build`, locates the binary, and passes it to the debug engine.
- **`thread <id>` command**: Switches the active REPL context to the named thread.
- **Threaded debuggee**: A variant of `rust_app_example` that spawns worker threads.
- **Tokio debuggee**: A variant of `rust_app_example` using `tokio::spawn` for async tasks.
- **`TreeStats`/`TreeError`**: Custom struct/enum added to `rust_app_example` for pretty-print testing.
- **Print configuration**: Session-level settings (`print-limit`, `print-depth`, `pretty-print`) that govern output verbosity.
- **Golden path snapshot**: A stored normalized output file used for regression comparison.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After implementation, the number of `#[ignore]` tests in `tests/big_test_plan.rs` that can be enabled drops from 90 to 0 (with any truly deprecated tests removed instead of re-ignored).
- **SC-002**: `cargo test --test big_test_plan` completes with 0 failures among the 90 formerly ignored tests when all features are implemented.
- **SC-003**: The `next` and `finish` commands each respond within 500 ms of being issued on the test machine.
- **SC-004**: `cargo debug` correctly builds and launches the project in under 30 seconds on a clean build.
- **SC-005**: All thread-related commands (`threads`, `thread <id>`, `bt`, `regs` on a worker thread) work correctly in the multi-threaded debuggee without deadlock or crash.
- **SC-006**: A 50-iteration debugging session (50 × break-hit + inspect) completes without observable memory growth in rde-cli.
- **SC-007**: After all features are delivered and all `#[ignore]` tags are removed, `cargo test --test big_test_plan` MUST complete with **0 ignored** tests and **0 failed** tests across the full suite (all 291 TCs).

---

## Assumptions

- The `rust_app_example` binary is the primary debuggee; no other target binary is needed for most tests.
- Windows 10/11 x64 is the only supported platform; no cross-platform behavior is required.
- Debug symbols (PDB) are always available for `rust_app_example` because it is built in debug mode.
- `next` and `finish` will be implemented using Win32 `SetThreadContext` + single-step or software breakpoint techniques, not source-level DWARF stepping (which is not in scope per the Engine-First constitution).
- The TUI implementation in `rde-tui` exists and only needs to be wired into the CLI launcher.
- The Tokio debuggee is a simple example (e.g., two `tokio::spawn` tasks) sufficient to exercise the `tasks` command.
- Performance tests use wall-clock timing measured by the test harness; no instrumentation is added to rde-cli itself.
- The golden path snapshot is generated once manually, committed, and then compared against on each run.
