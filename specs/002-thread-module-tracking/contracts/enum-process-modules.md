# Contract: `EnumProcessModules`

## Pre-conditions
- Process handle must have `PROCESS_QUERY_INFORMATION | PROCESS_VM_READ`.
- `lpcbNeeded` must point to writable `DWORD`.
- `cb` must be at least `sizeof(HMODULE)`.

## Post-conditions (Ok)
- `lphModule` array is populated with `HMODULE` handles for all loaded modules.
- `lpcbNeeded` contains the total bytes required for all modules.
- If `cb` is too small, returns `TRUE` but only fills available space; caller must retry.

## Post-conditions (Err)
- `ERROR_PARTIAL_COPY`: Process is 32-bit and caller is 64-bit (or vice versa). Use
  `EnumProcessModulesEx` with `LIST_MODULES_32BIT` / `LIST_MODULES_64BIT` if cross-bitness
  support is needed.
- `ERROR_ACCESS_DENIED`: Process handle lacks required access rights.

## Invariant
- This is a snapshot API. Modules loaded or unloaded after the call are not reflected.
- Use only as a fallback / initial scan; primary module tracking MUST be event-driven via
  `LOAD_DLL_DEBUG_EVENT` and `UNLOAD_DLL_DEBUG_EVENT`.
- Combine with `K32GetModuleInformation` to retrieve base address and size for each module.
