# Research: Thread and Module Tracking

**Feature**: Thread and Module Tracking (002)
**Date**: 2026-05-28

---

## Decision 1: Thread Handle Caching Strategy

**Decision**: Cache `hThread` handles from `CREATE_THREAD_DEBUG_EVENT` in the engine's `Thread`
registry, and close them on `EXIT_THREAD_DEBUG_EVENT` or session end.

**Rationale**:
- The current `rde-win32/src/thread.rs` opens and immediately closes a thread handle on every
  `get_context`, `set_context`, `suspend`, and `resume` call. This is inefficient and risks handle
  leaks under error conditions.
- The Win32 debug event `CREATE_THREAD_DEBUG_EVENT` provides a valid `hThread` handle that the
  debugger can retain.
- Caching satisfies FR-012 (handle caching) and SC-06 (no handle leaks over long sessions).

**Alternatives considered**:
- **Keep opening/closing**: Rejected because it violates FR-012 and is measurably slower under
  frequent thread operations.
- **Lazy caching (open on first use, close on session end)**: Rejected because `OpenThread` can
  fail if the thread exits between list and use. The debug event's `hThread` is guaranteed valid
  at creation time.

---

## Decision 2: Module Name Resolution from LOAD_DLL_DEBUG_EVENT

**Decision**: Use a two-tier approach:
1. Primary: `GetMappedFileNameW` with the process handle and `lpBaseOfDll` from the debug event.
2. Fallback: Read the `lpImageName` pointer from `LOAD_DLL_DEBUG_INFO` via `ReadProcessMemory`
   (it points to a `PWSTR` in the target process address space).

**Rationale**:
- `GetMappedFileNameW` returns the NT device path (`\Device\HarddiskVolume...`) and is reliable
  for any mapped module. It can be translated to a DOS path via `QueryDosDevice` loop.
- The `lpImageName` field in `LOAD_DLL_DEBUG_INFO` is documented by Microsoft as potentially
  `NULL` or invalid in certain attach scenarios, making it unsuitable as a primary source.
- `ReadProcessMemory` to read the `PWSTR` itself requires dereferencing a pointer in the target
  process, which adds complexity.

**Alternatives considered**:
- **Use `hFile` from `LOAD_DLL_DEBUG_INFO` + `GetFinalPathNameByHandleW`**: Rejected because
  `hFile` may be `NULL` or closed by the system before the debugger processes the event.
- **Post-load `EnumProcessModules` scan**: Rejected because it is a polling pattern; event-driven
  `GetMappedFileNameW` is more immediate and aligns with the REPL-First architecture.

---

## Decision 3: Symbol Engine Integration on Module Load

**Decision**: Call `SymLoadModuleExW` (or the narrow `SymLoadModuleEx` if Unicode variant is
unavailable in the loaded DbgHelp) in `DbgHelpSymbolEngine::load_module` when the engine
processes a `ModuleLoaded` event.

**Rationale**:
- DbgHelp's `SymInitializeW` with `fInvadeProcess = TRUE` attempts to load symbols for modules
  already loaded at initialization time, but it does NOT automatically load symbols for modules
  loaded after initialization.
- Calling `SymLoadModuleEx` explicitly ensures that stack traces and symbol resolution work for
  dynamically loaded DLLs (e.g., plugins, COM objects, late-bound libraries).
- The `load_module` stub in `rde-symbols/src/dbghelp.rs` already exists; this decision turns it
  into a real implementation without changing the `SymbolEngine` trait signature.

**Alternatives considered**:
- **Set symbol search path to include all possible DLL directories and rely on auto-load**: Rejected
  because it is unreliable for DLLs loaded from temporary or non-standard paths.
- **Defer symbol loading until first address resolution in the module**: Rejected because it would
  cause a noticeable pause on the first stack trace after a DLL loads, violating the REPL responsiveness
  requirement.

---

## Decision 4: Selected Thread Fallback on Thread Exit

**Decision**: When the selected thread exits, automatically select the oldest remaining running
thread (lowest `ThreadId` or earliest creation timestamp). If no threads remain, set selected to
`None` and require explicit selection when a new thread appears.

**Rationale**:
- The user must always have a valid context for `regs`, `bt`, and `step`. Silently switching to
  another thread is preferable to failing the command.
- Using the oldest thread is deterministic and predictable.
- If no threads exist (transient state after process creation before first thread event), commands
  that need a thread context return a clear error.

**Alternatives considered**:
- **Keep selected_thread pointing to exited thread and error on use**: Rejected because it creates
  a poor user experience — every command would fail until the user manually selects another thread.
- **Select the newest thread**: Rejected because it is less predictable; the main thread is usually
  the oldest and is the most common context of interest.

---

## Decision 5: UNLOAD_DLL_DEBUG_EVENT Handling

**Decision**: Add `UNLOAD_DLL_DEBUG_EVENT` handling to the debug loop, send a new
`EngineEvent::ModuleUnloaded { base: u64 }`, and remove the module from the engine registry.

**Rationale**:
- The spec requires tracking module descarga (FR-008). The Win32 debug API generates this event
  when `FreeLibrary` is called or the process unmaps a DLL.
- The event provides `lpBaseOfDll`, which is the same key used in the module registry.
- Removing the module keeps the registry accurate and prevents stale symbol lookups.

**Alternatives considered**:
- **Keep unloaded modules in registry with a "dead" flag**: Rejected because it complicates the
  module list display and symbol resolution without clear user benefit. Historical module state
  is not a priority for the MVP.
