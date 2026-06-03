//! Memory read/write wrappers.

use rde_core::{DebugError, ProcessHandle};
use tracing::instrument;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{FlushInstructionCache, ReadProcessMemory};
use windows::Win32::System::Memory::{VirtualProtectEx, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS};

/// Read memory from the target process.
#[instrument]
pub fn read_memory(handle: &ProcessHandle, address: u64, size: usize, bytes: &mut [u8]) -> Result<usize, DebugError> {
    let h = HANDLE(handle.process_handle.0 as isize);
    let mut read = 0usize;

    // SAFETY: h is a valid process handle with debug privileges.
    // address and bytes are valid for the operation.
    // GitHub issue: TODO(rde#1)
    let result = unsafe {
        ReadProcessMemory(
            h,
            address as *const std::ffi::c_void,
            bytes.as_mut_ptr() as *mut std::ffi::c_void,
            size,
            Some(&mut read),
        )
    };

    if let Err(e) = result {
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("ReadProcessMemory failed at 0x{address:x}"),
        });
    }
    Ok(read)
}

/// Write memory to the target process.
#[instrument]
pub fn write_memory(handle: &ProcessHandle, address: u64, bytes: &[u8]) -> Result<(), DebugError> {
    let h = HANDLE(handle.process_handle.0 as isize);

    // Temporarily change protection to allow writing (especially for code pages)
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    // SAFETY: h is a valid process handle.
    // GitHub issue: TODO(rde#1)
    let prot_result = unsafe {
        VirtualProtectEx(
            h,
            address as *const std::ffi::c_void,
            bytes.len(),
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
    };
    if let Err(e) = prot_result {
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("VirtualProtectEx failed at 0x{address:x}"),
        });
    }

    let mut written = 0usize;
    // SAFETY: h is a valid process handle; memory protection was just updated.
    // GitHub issue: TODO(rde#1)
    let write_result = unsafe {
        windows::Win32::System::Diagnostics::Debug::WriteProcessMemory(
            h,
            address as *mut std::ffi::c_void,
            bytes.as_ptr() as *const std::ffi::c_void,
            bytes.len(),
            Some(&mut written),
        )
    };

    // Restore original protection regardless of write result
    let _ = unsafe {
        VirtualProtectEx(
            h,
            address as *const std::ffi::c_void,
            bytes.len(),
            old_protect,
            &mut old_protect,
        )
    };

    if let Err(e) = write_result {
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("WriteProcessMemory failed at 0x{address:x}"),
        });
    }

    // INVARIANT: Always call FlushInstructionCache after WriteProcessMemory on a code page.
    // Without this, the CPU may execute the cached (non-patched) instruction on multi-core systems.
    // GitHub issue: TODO(rde#step-over-contract)
    let _ = unsafe { FlushInstructionCache(h, Some(address as *const std::ffi::c_void), bytes.len()) };

    Ok(())
}
