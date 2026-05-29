# RDE Guidelines & API Documentation

Complete user and API documentation for the Rust Debugger Engine (RDE) workspace crates.

---

## Quick Navigation

| Document | Scope |
|----------|-------|
| [debugger-usage.md](debugger-usage.md) | End-to-end user guide: launch, breakpoints, inspection, cargo |
| [rde-cli.md](rde-cli.md) | CLI reference, subcommands, flags, examples |
| [rde-repl.md](rde-repl.md) | REPL commands, configuration, pretty-print flags |
| [rde-core.md](rde-core.md) | Engine API, events, commands, extensions |
| [rde-orchestrator.md](rde-orchestrator.md) | High-level orchestration API |
| [rde-pretty-print.md](rde-pretty-print.md) | Pretty printer API, registry, custom printers |
| [rde-tokio.md](rde-tokio.md) | Tokio task inspection API |
| [rde-cargo.md](rde-cargo.md) | Cargo metadata, target resolution, build API |

---

## Architecture Overview

```
User Layer
  ├── rde-cli      (binary entry point)
  └── rde-repl     (interactive command loop)

Orchestration Layer
  └── rde-orchestrator  (coordinates engine + extensions)

Engine Layer
  └── rde-core     (debug engine, state machine, channels)

Extension Layer
  ├── rde-pretty-print  (Rust value formatting)
  ├── rde-tokio         (async task inspection)
  └── rde-cargo         (Cargo project integration)

Backend Layer
  ├── rde-win32    (Win32 Debug API)
  ├── rde-symbols  (PDB / DbgHelp)
  └── rde-breakpoint  (breakpoint manager)
```

All communication between layers uses async message passing (`tokio::sync::mpsc`).
