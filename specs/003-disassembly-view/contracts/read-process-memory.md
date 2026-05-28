# Contract: ReadProcessMemory

**Feature**: 003-disassembly-view
**API**: `ReadProcessMemory` (Win32)
**Crate**: `rde-win32`
**Date**: 2026-05-28

## Pre-conditions

1. `hProcess` é um handle válido para o processo debuggee, obtido via `CreateProcessW` (com `DEBUG_PROCESS`) ou `OpenProcess` (para attach), e ainda não foi fechado.
2. O handle possui direito de acesso `PROCESS_VM_READ`.
3. `lpBaseAddress` é um endereço virtual de 64 bits dentro do espaço de endereço do processo target. Não é necessário alinhamento (x86-64 suporta leitura byte-a-byte).
4. `nSize` > 0 e ≤ 4096 (invariante imposto pelo feature 003; o engine nunca solicita leitura maior que 4KB para disassembly).
5. `lpBuffer` aponta para memória alocada no processo debugger com tamanho ≥ `nSize`.
6. `lpNumberOfBytesRead` aponta para uma variável `usize` válida.

## Post-conditions on success (`Ok`)

1. `lpBuffer[0..*lpNumberOfBytesRead]` contém os bytes lidos do espaço de endereço do processo debuggee.
2. `*lpNumberOfBytesRead` ≤ `nSize`.
3. Se `*lpNumberOfBytesRead` < `nSize`, a memória a partir de `lpBaseAddress + *lpNumberOfBytesRead` não é legível (ex: página não commitada, guard page, ou fim do allocation).
4. A memória do processo debuggee não é modificada.

## Post-conditions on failure (`Err`)

1. `GetLastError()` retorna um código não-zero.
2. `lpBuffer` conteúdo é **indeterminado** — não deve ser usado.
3. `*lpNumberOfBytesRead` é **indeterminado** — não deve ser confiado.
4. Causas comuns:
   - `ERROR_INVALID_HANDLE` (6): handle inválido ou sem `PROCESS_VM_READ`
   - `ERROR_ACCESS_DENIED` (5): sem permissão para ler memória do target
   - `ERROR_PARTIAL_COPY` (299): leitura cruzou região não-mapeada; neste caso, *nenhum* byte foi copiado (diferente de sucesso parcial)
   - `ERROR_NOACCESS` (998): endereço não acessível

## Invariants

- **INVARIANT**: O engine nunca chama `ReadProcessMemory` com `nSize > 4096` durante disassembly.  
  **VIOLATION**: Leituras massivas podem causar stalls no debug loop e violam FR-013. *(Issue: N/A — garantido por construção no módulo `disasm`)*
- **INVARIANT**: `lpBuffer` é alocado no stack do debugger (array de 4096 bytes) ou em `Vec<u8>` com lifetime limitada ao escopo da função wrapper.  
  **VIOLATION**: Buffer vazado para fora do escopo causaria use-after-free ou corrupção de heap.
- **INVARIANT**: O wrapper Rust verifica `*lpNumberOfBytesRead` antes de retornar `Ok`; se for 0 e `nSize > 0`, retorna `Err` com `GetLastError()`.  
  **VIOLATION**: Retornar `Ok` com 0 bytes lido confundiria o disassembler (Capstone receberia slice vazio).
- **INVARIANT**: `ReadProcessMemory` é chamada apenas quando o processo debuggee está parado (em estado de debug event) ou quando o debugger tem `PROCESS_VM_READ`. Nunca durante execução livre sem direito adequado.  
  **VIOLATION**: Tentativa de leitura em processo running sem direitos corretos gera `ERROR_ACCESS_DENIED`.

## Type-system distinctions

- O wrapper Rust retorna `Result<Vec<u8>, DebugError>`, nunca expõe `BOOL` ou `GetLastError` raw ao caller.
- Diferença entre "sucesso parcial" (`*lpNumberOfBytesRead` < `nSize`, mas > 0) e "falha total" (`BOOL == 0`) é preservada: sucesso parcial → `Ok(vec_com_bytes_lidos)`; falha total → `Err(DebugError::ReadMemoryFailed { address, source })`.

## Usage in this feature

```rust
// INVARIANT: nSize <= 4096 (enforced by caller)
// INVARIANT: process_handle is valid and has PROCESS_VM_READ
let mut buf = vec![0u8; nSize];
let mut read = 0usize;
let ok = unsafe {
    ReadProcessMemory(
        handle.0,           // hProcess
        address as *const _, // lpBaseAddress
        buf.as_mut_ptr() as *mut _, // lpBuffer
        nSize,              // nSize
        &mut read,          // lpNumberOfBytesRead
    )
};
if ok == FALSE {
    // VIOLATION: buf and read are undefined here
    return Err(DebugError::ReadMemoryFailed { address, source: GetLastError().into() });
}
// INVARIANT: read > 0 or nSize == 0
if read == 0 && nSize > 0 {
    return Err(DebugError::ReadMemoryFailed { address, source: std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "ReadProcessMemory returned 0 bytes") });
}
buf.truncate(read);
Ok(buf)
```
