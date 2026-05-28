# Contract: `GetMappedFileNameW`

## Pre-conditions
- Process handle must have `PROCESS_QUERY_INFORMATION` access.
- `lpv` must be a valid memory address mapped to a file in the target process.
- Buffer (`lpFilename`) must be writable and at least `nSize` wide characters.

## Post-conditions (Ok)
- Buffer contains the NT object path of the mapped file (e.g., `\Device\HarddiskVolume4\Windows\System32\kernel32.dll`).
- Return value is the length of the string in characters, excluding null terminator.

## Post-conditions (Err)
- `ERROR_INVALID_PARAMETER`: `lpv` is not a valid mapped address.
- `ERROR_INSUFFICIENT_BUFFER`: Buffer too small; caller should retry with larger buffer.
- Buffer contents are UNDEFINED on error.

## Invariant
- The returned path is an NT path, NOT a DOS path. To display to users, translate via
  `QueryDosDevice` for each drive letter.
- This function is safe to call from the debug loop thread because it performs no blocking I/O
  outside the kernel.
