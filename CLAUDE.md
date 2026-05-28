# RDE Project Context

This file provides runtime guidance for AI agents working on the Rust Debugger Engine (RDE)
project.

<!-- SPECKIT START -->

## Active Plan

- **Feature**: 004-tui-interface
- **Spec**: [specs/004-tui-interface/spec.md](specs/004-tui-interface/spec.md)
- **Plan**: [specs/004-tui-interface/plan.md](specs/004-tui-interface/plan.md)
- **Plan**: [specs/002-thread-module-tracking/plan.md](specs/002-thread-module-tracking/plan.md)

## Constitution

See `.specify/memory/constitution.md` (v4.0.0).

Key non-negotiables:
- Engine-First: no LLDB/GDB/wrappers. Native Win32 Debug API only.
- Crate-First: every capability is a standalone crate in `crates/`.
- REPL-First: async message passing; UI never blocks.
- Windows-Native: Windows 10/11 x64 only. No cross-platform abstractions in core.
- Rust Safety: `unsafe` only in `rde-win32` with three-condition proof. No Python/PyO3.

## Workspace Layout

```
crates/
  rde-core/         — Traits, types, engine state machine, channels
  rde-win32/        — Win32 API backend (unsafe zone)
  rde-symbols/      — DbgHelp integration, demangling
  rde-breakpoint/   — Breakpoint manager
  rde-repl/         — Command parser, REPL loop
  rde-cli/          — Binary entry point
```

## Commands

Use speckit commands for workflow:
- `/speckit-specify` — new feature spec
- `/speckit-plan` — implementation planning
- `/speckit-tasks` — task generation
- `/speckit-implement` — implementation

<!-- SPECKIT END -->

## General Guidance

- Prefer simplicity. The goal is a hackable debugger, not a second LLDB.
- All public APIs must have doc comments and usage examples.
- Write tests first (Red-Green-Refactor).
- Windows-only. Do not introduce Linux/macOS code into `rde-core` or `rde-win32`.
