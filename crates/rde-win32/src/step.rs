//! Step-over and step-out primitives using temporary INT3 breakpoints.
//!
//! # Protocol (Step-Over)
//! 1. Read RIP via GetThreadContext.
//! 2. Read 16 bytes at RIP via ReadProcessMemory.
//! 3. Decode instruction length L using Capstone.
//! 4. Save original byte at RIP+L.
//! 5. Write 0xCC at RIP+L (plant temp BP).
//! 6. FlushInstructionCache(RIP+L, 1).
//! 7. ContinueDebugEvent → WaitForDebugEventEx.
//! 8. On EXCEPTION_BREAKPOINT at RIP+L: restore byte, decrement RIP, SetThreadContext.
//!
//! # Safety
//! All `unsafe` blocks in this module require a valid debug process handle with
//! PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION access, which is
//! granted automatically to the process that called CreateProcessW with DEBUG_PROCESS.
//! GitHub issue: TODO(rde#step-over-contract)

use capstone::prelude::*;
use rde_core::{DebugError, ProcessHandle, ThreadId};
use rde_symbols::{DbgHelpSymbolEngine, SymbolEngine};
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{
    FlushInstructionCache, ReadProcessMemory, WriteProcessMemory,
};
use windows::Win32::System::Memory::{VirtualProtectEx, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS};

/// Plant a temporary INT3 breakpoint at `address` in the target process.
///
/// Returns the original byte at that address, which the caller MUST save and pass
/// to `restore_temp_breakpoint` after the breakpoint fires.
///
/// # Invariants
/// - The caller must call `FlushInstructionCache` after this returns (done internally).
/// - The caller must save the returned `u8` and restore it before resuming.
/// - MUST NOT be called while the target process is running.
pub fn plant_temp_breakpoint(handle: &ProcessHandle, address: u64) -> Result<u8, DebugError> {
    let h = HANDLE(handle.process_handle.0 as isize);

    // 1. Read original byte
    let mut orig = [0u8; 1];
    let mut bytes_read = 0usize;
    // SAFETY: h is a valid debug process handle; address is a virtual address in the target.
    // ReadProcessMemory is safe to call on code pages from the debug process.
    // GitHub issue: TODO(rde#step-over-contract)
    let ok = unsafe {
        ReadProcessMemory(
            h,
            address as *const std::ffi::c_void,
            orig.as_mut_ptr() as *mut std::ffi::c_void,
            1,
            Some(&mut bytes_read),
        )
    };
    if ok.is_err() || bytes_read != 1 {
        return Err(DebugError::Win32Error {
            code: 0,
            message: format!("ReadProcessMemory failed reading original byte at 0x{address:x}"),
        });
    }

    // 2. Make page writable, write 0xCC, restore protection
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    // SAFETY: VirtualProtectEx on a debug process handle — we have the required access rights.
    // GitHub issue: TODO(rde#step-over-contract)
    let _ = unsafe {
        VirtualProtectEx(
            h,
            address as *const std::ffi::c_void,
            1,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
    };

    let int3 = [0xCCu8];
    let mut bytes_written = 0usize;
    // SAFETY: h is a valid debug handle; memory was just made writable above.
    // INVARIANT: Save original byte BEFORE calling WriteProcessMemory.
    // VIOLATION: If we hadn't read orig first, restoration after the temp BP fires
    //            would be impossible — the address would be permanently INT3.
    // GitHub issue: TODO(rde#step-over-contract)
    let write_ok = unsafe {
        WriteProcessMemory(
            h,
            address as *mut std::ffi::c_void,
            int3.as_ptr() as *const std::ffi::c_void,
            1,
            Some(&mut bytes_written),
        )
    };

    // Restore page protection
    let _ = unsafe {
        VirtualProtectEx(
            h,
            address as *const std::ffi::c_void,
            1,
            old_protect,
            &mut old_protect,
        )
    };

    if write_ok.is_err() || bytes_written != 1 {
        return Err(DebugError::Win32Error {
            code: 0,
            message: format!("WriteProcessMemory failed planting INT3 at 0x{address:x}"),
        });
    }

    // 3. Flush instruction cache
    // INVARIANT: Always call FlushInstructionCache after WriteProcessMemory on a code page.
    // VIOLATION: Without cache flush, the CPU may execute the cached (non-patched) instruction,
    //            causing the temp BP to never fire.
    // GitHub issue: TODO(rde#step-over-contract)
    let _ = unsafe { FlushInstructionCache(h, Some(address as *const std::ffi::c_void), 1) };

    Ok(orig[0])
}

/// Restore the original byte at `address` that was overwritten by a temp breakpoint.
///
/// # Invariants
/// - MUST be called BEFORE SetThreadContext(Rip-1).
/// - VIOLATION: If original byte is still 0xCC when execution resumes, the CPU hits INT3
///   again → infinite breakpoint loop.
/// GitHub issue: TODO(rde#step-over-contract)
pub fn restore_temp_breakpoint(
    handle: &ProcessHandle,
    address: u64,
    original_byte: u8,
) -> Result<(), DebugError> {
    let h = HANDLE(handle.process_handle.0 as isize);

    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    // SAFETY: Debug process handle with required access.
    // GitHub issue: TODO(rde#step-over-contract)
    let _ = unsafe {
        VirtualProtectEx(
            h,
            address as *const std::ffi::c_void,
            1,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
    };

    let buf = [original_byte];
    let mut written = 0usize;
    // SAFETY: h is a valid debug handle; memory was just made writable.
    // GitHub issue: TODO(rde#step-over-contract)
    let ok = unsafe {
        WriteProcessMemory(
            h,
            address as *mut std::ffi::c_void,
            buf.as_ptr() as *const std::ffi::c_void,
            1,
            Some(&mut written),
        )
    };

    let _ = unsafe {
        VirtualProtectEx(
            h,
            address as *const std::ffi::c_void,
            1,
            old_protect,
            &mut old_protect,
        )
    };

    if ok.is_err() || written != 1 {
        return Err(DebugError::Win32Error {
            code: 0,
            message: format!("WriteProcessMemory failed restoring byte at 0x{address:x}"),
        });
    }

    // INVARIANT: Always call FlushInstructionCache after WriteProcessMemory on a code page.
    // GitHub issue: TODO(rde#step-over-contract)
    let _ = unsafe { FlushInstructionCache(h, Some(address as *const std::ffi::c_void), 1) };

    Ok(())
}

/// Compute the address for a step-over temp BP by decoding the instruction at `rip`.
///
/// Uses Capstone to decode the current instruction and returns `rip + instruction_length`.
/// Falls back to `rip + 1` if decoding fails (safe conservative choice).
pub fn next_instruction_address(handle: &ProcessHandle, rip: u64) -> Result<u64, DebugError> {
    let h = HANDLE(handle.process_handle.0 as isize);

    // Read up to 16 bytes at RIP for instruction decoding
    let mut buf = [0u8; 16];
    let mut bytes_read = 0usize;
    // SAFETY: h is a valid debug process handle; rip is the current instruction pointer.
    // GitHub issue: TODO(rde#step-over-contract)
    let ok = unsafe {
        ReadProcessMemory(
            h,
            rip as *const std::ffi::c_void,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
            16,
            Some(&mut bytes_read),
        )
    };

    if ok.is_err() || bytes_read == 0 {
        return Err(DebugError::Win32Error {
            code: 0,
            message: format!("ReadProcessMemory failed reading instruction bytes at 0x{rip:x}"),
        });
    }

    // Decode with Capstone (x86-64 mode)
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .detail(false)
        .build()
        .map_err(|e| DebugError::Internal(format!("Capstone init failed: {e}")))?;

    let valid_bytes = &buf[..bytes_read];
    let insns = cs.disasm_count(valid_bytes, rip, 1)
        .map_err(|e| DebugError::Internal(format!("Capstone disasm failed: {e}")))?;

    if let Some(insn) = insns.iter().next() {
        let len = insn.bytes().len() as u64;
        if len > 0 {
            return Ok(rip + len);
        }
    }

    // Fallback: advance 1 byte (conservative)
    tracing::warn!("Capstone failed to decode instruction at 0x{rip:x}, falling back to rip+1");
    Ok(rip + 1)
}

/// Read the return address for step-out using the symbol engine's stack walker.
///
/// This correctly handles functions where RSP has been adjusted by the prologue,
/// unlike the naive `[RSP]` read which is only valid at function entry.
pub fn read_return_address(
    thread_id: ThreadId,
    symbol_engine_arc: &Arc<Mutex<Option<DbgHelpSymbolEngine>>>,
) -> Result<u64, DebugError> {
    let engine = symbol_engine_arc.lock().map_err(|e| {
        DebugError::Internal(format!("Failed to lock symbol engine: {e}"))
    })?;
    let engine = engine.as_ref().ok_or_else(|| {
        DebugError::Internal("Symbol engine not initialized".into())
    })?;

    let frames = engine.walk_stack(thread_id).map_err(|e| {
        DebugError::Internal(format!("Stack walk failed: {e}"))
    })?;

    frames
        .first()
        .map(|f| f.return_address)
        .ok_or_else(|| DebugError::Internal("Could not find return address on stack".into()))
}
