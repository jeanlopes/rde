# Contract: `SymLoadModuleEx`

## Pre-conditions
- `SymInitializeW` (or equivalent) must have been called for the process handle.
- `BaseOfDll` must be the load address reported by `LOAD_DLL_DEBUG_EVENT`.
- If `SizeOfDll` is zero, DbgHelp attempts to read the PE headers to determine size.

## Post-conditions (Ok)
- Module is registered with DbgHelp for symbol resolution.
- Returns the base address of the loaded module (non-zero).
- Subsequent `SymFromAddr` calls within this module's address range will resolve symbols.

## Post-conditions (Err)
- Returns zero on failure.
- `GetLastError` may return `ERROR_NOT_SUPPORTED` if no matching PDB is found.
- Existing symbol table is NOT corrupted on failure; the module is simply not added.

## Invariant
- Must be called for every module loaded AFTER `SymInitializeW`. Modules present at
  initialization time are auto-loaded if `fInvadeProcess` was `TRUE`.
- Calling `SymLoadModuleEx` for a module already loaded is a no-op (returns existing base).
- Must be paired with `SymUnloadModule64` when the module is unloaded, to free symbol resources.
