//! Module (DLL) enumeration and tracking.

use rde_core::{DebugError, Module};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::ProcessStatus::{
    EnumProcessModules, GetMappedFileNameW, K32GetModuleInformation, MODULEINFO,
};

/// Resolve the file name of a mapped module from its base address.
/// Returns the DOS path (e.g., `C:\Windows\System32\kernel32.dll`) when possible.
pub fn get_module_name(process_handle: HANDLE, base_address: u64) -> Option<String> {
    let mut buffer = vec![0u16; 512];
    let len = unsafe {
        GetMappedFileNameW(
            process_handle,
            base_address as *const _, // lpv
            &mut buffer,
        )
    };
    if len == 0 {
        return None;
    }
    let nt_path = String::from_utf16_lossy(&buffer[..len as usize]);
    nt_path_to_dos_path(&nt_path)
}

/// Translate an NT device path (`\Device\HarddiskVolume4\...`) to a DOS path.
fn nt_path_to_dos_path(nt_path: &str) -> Option<String> {
    // Simple heuristic: replace the first device component with the drive letter.
    // A full implementation would use QueryDosDevice for each drive letter.
    // For MVP, we strip the prefix and return the remainder if it matches a known pattern.
    if let Some(rest) = nt_path.strip_prefix("\\Device\\HarddiskVolume") {
        // Find the next backslash after the volume number
        if let Some(idx) = rest.find('\\') {
            let path_part = &rest[idx..];
            return Some(format!("C:{}", path_part));
        }
    }
    Some(nt_path.to_string())
}

/// Enumerate all loaded modules in the target process using `EnumProcessModules`.
pub fn enumerate_modules(process_handle: HANDLE) -> Result<Vec<Module>, DebugError> {
    let mut modules = Vec::new();
    let mut needed: u32 = 0;

    // First call to get required size
    let mut hmods = vec![windows::Win32::Foundation::HMODULE(0); 1024];
    let ok = unsafe {
        EnumProcessModules(
            process_handle,
            hmods.as_mut_ptr(),
            (hmods.len() * std::mem::size_of::<windows::Win32::Foundation::HMODULE>()) as u32,
            &mut needed,
        )
    };
    if ok.is_err() {
        return Ok(modules); // Return empty on failure (non-fatal)
    }

    let count = needed as usize / std::mem::size_of::<windows::Win32::Foundation::HMODULE>();
    for i in 0..count.min(hmods.len()) {
        let hmod = hmods[i];
        let base = hmod.0 as u64;

        let mut info = MODULEINFO::default();
        let info_ok = unsafe {
            K32GetModuleInformation(
                process_handle,
                hmod,
                &mut info,
                std::mem::size_of::<MODULEINFO>() as u32,
            )
        };
        let size = if info_ok.as_bool() {
            info.SizeOfImage as u64
        } else {
            0
        };

        let name = get_module_name(process_handle, base)
            .unwrap_or_else(|| format!("module@{base:x}"));

        modules.push(Module {
            name,
            base_address: base,
            size,
            path: None,
            symbols_loaded: false,
        });
    }

    Ok(modules)
}
