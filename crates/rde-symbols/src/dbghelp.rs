//! DbgHelp.dll dynamic loading and symbol engine implementation.

use crate::SymbolEngine;
use libloading::{Library, Symbol};
use rde_core::{DebugError, StackFrame, SymbolInfo};
use std::ffi::{c_void, CStr};
use std::os::windows::ffi::OsStrExt;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// FFI types
// ---------------------------------------------------------------------------

type DWORD = u32;
type DWORD64 = u64;
type BOOL = i32;
type HANDLE = isize;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ADDRESS64 {
    offset: DWORD64,
    segment: u16,
    mode: u32, // ADDRESS_MODE
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct KDHELP64 {
    thread: DWORD64,
    th_callback_stack: DWORD,
    th_callback_bstore: DWORD,
    next_callback: DWORD,
    frame_pointer: DWORD,
    ki_call_user_mode: DWORD64,
    ke_user_callback_dispatcher: DWORD64,
    system_range_start: DWORD64,
    ki_user_exception_dispatcher: DWORD64,
    stack_base: DWORD64,
    stack_limit: DWORD64,
    reserved: [DWORD64; 5],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct STACKFRAME64 {
    addr_pc: ADDRESS64,
    addr_return: ADDRESS64,
    addr_frame: ADDRESS64,
    addr_stack: ADDRESS64,
    addr_bstore: ADDRESS64,
    func_table_entry: *mut c_void,
    params: [DWORD64; 4],
    far_: BOOL,
    virtual_: BOOL,
    reserved: [DWORD64; 3],
    kdhelp: KDHELP64,
}

#[repr(C)]
#[derive(Debug)]
struct SYMBOL_INFO {
    size_of_struct: DWORD,
    type_index: DWORD,
    reserved: [DWORD64; 2],
    index: DWORD,
    size: DWORD,
    mod_base: DWORD64,
    flags: DWORD,
    value: DWORD64,
    address: DWORD64,
    register: DWORD,
    scope: DWORD,
    tag: DWORD,
    name_len: DWORD,
    max_name_len: DWORD,
    name: [u8; 1],
}

#[repr(C)]
#[derive(Debug)]
struct IMAGEHLP_LINE64 {
    size_of_struct: DWORD,
    key: *mut c_void,
    line_number: DWORD,
    file_name: *mut u8,
    address: DWORD64,
}

// ---------------------------------------------------------------------------
// Function pointer types
// ---------------------------------------------------------------------------

type SymInitializeWFn = unsafe extern "system" fn(HANDLE, *const u16, BOOL) -> BOOL;
type SymCleanupFn = unsafe extern "system" fn(HANDLE) -> BOOL;
type SymFromAddrWFn = unsafe extern "system" fn(HANDLE, DWORD64, *mut DWORD64, *mut SYMBOL_INFO) -> BOOL;
type SymGetLineFromAddrW64Fn = unsafe extern "system" fn(HANDLE, DWORD64, *mut DWORD, *mut IMAGEHLP_LINE64) -> BOOL;
type SymLoadModuleExWFn = unsafe extern "system" fn(
    HANDLE,
    HANDLE,
    *const u16,
    *const u16,
    DWORD64,
    DWORD,
    *mut c_void,
    DWORD,
) -> DWORD64;
type SymUnloadModule64Fn = unsafe extern "system" fn(HANDLE, DWORD64) -> BOOL;
type StackWalk64Fn = unsafe extern "system" fn(
    DWORD,
    HANDLE,
    HANDLE,
    *mut STACKFRAME64,
    *mut c_void, // CONTEXT
    Option<unsafe extern "system" fn(HANDLE, DWORD64, *mut c_void, DWORD, *mut DWORD) -> BOOL>,
    Option<unsafe extern "system" fn(HANDLE, DWORD64, *mut c_void) -> DWORD64>,
    Option<unsafe extern "system" fn(HANDLE, DWORD, *mut c_void, *mut DWORD64) -> DWORD64>,
    Option<unsafe extern "system" fn(DWORD, *mut STACKFRAME64, *mut c_void, DWORD) -> BOOL>,
) -> BOOL;

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Dynamic loader for DbgHelp APIs.
pub struct DbgHelpLoader {
    _lib: Library,
    sym_initialize_w: Symbol<'static, SymInitializeWFn>,
    sym_cleanup: Symbol<'static, SymCleanupFn>,
    sym_from_addr_w: Symbol<'static, SymFromAddrWFn>,
    sym_get_line_from_addr_w64: Symbol<'static, SymGetLineFromAddrW64Fn>,
    sym_load_module_ex_w: Symbol<'static, SymLoadModuleExWFn>,
    sym_unload_module64: Symbol<'static, SymUnloadModule64Fn>,
    #[allow(dead_code)]
    stack_walk64: Symbol<'static, StackWalk64Fn>,
}

impl DbgHelpLoader {
    pub fn new() -> Result<Self, DebugError> {
        // Load DbgHelp.dll from system directory
        let lib = unsafe {
            Library::new("DbgHelp.dll").map_err(|e| DebugError::Internal(format!("Failed to load DbgHelp.dll: {e}")))?
        };

        // Leak the library so symbols live for 'static.
        // This is acceptable for a debugger that loads DbgHelp once per session.
        let lib = Box::leak(Box::new(lib));

        let sym_initialize_w = unsafe {
            lib.get(b"SymInitializeW\0")
                .map_err(|e| DebugError::Internal(format!("SymInitializeW not found: {e}")))?
        };
        let sym_cleanup = unsafe {
            lib.get(b"SymCleanup\0")
                .map_err(|e| DebugError::Internal(format!("SymCleanup not found: {e}")))?
        };
        let sym_from_addr_w = unsafe {
            lib.get(b"SymFromAddrW\0")
                .map_err(|e| DebugError::Internal(format!("SymFromAddrW not found: {e}")))?
        };
        let sym_get_line_from_addr_w64 = unsafe {
            lib.get(b"SymGetLineFromAddrW64\0")
                .map_err(|e| DebugError::Internal(format!("SymGetLineFromAddrW64 not found: {e}")))?
        };
        let stack_walk64 = unsafe {
            lib.get(b"StackWalk64\0")
                .map_err(|e| DebugError::Internal(format!("StackWalk64 not found: {e}")))?
        };
        let sym_load_module_ex_w = unsafe {
            lib.get(b"SymLoadModuleExW\0")
                .map_err(|e| DebugError::Internal(format!("SymLoadModuleExW not found: {e}")))?
        };
        let sym_unload_module64 = unsafe {
            lib.get(b"SymUnloadModule64\0")
                .map_err(|e| DebugError::Internal(format!("SymUnloadModule64 not found: {e}")))?
        };

        Ok(Self {
            _lib: unsafe { Library::new("DbgHelp.dll").unwrap() }, // placeholder, not used
            sym_initialize_w,
            sym_cleanup,
            sym_from_addr_w,
            sym_get_line_from_addr_w64,
            sym_load_module_ex_w,
            sym_unload_module64,
            stack_walk64,
        })
    }
}

// ---------------------------------------------------------------------------
// Symbol Engine
// ---------------------------------------------------------------------------

/// Symbol engine backed by Windows DbgHelp.
pub struct DbgHelpSymbolEngine {
    process_handle: HANDLE,
    loader: DbgHelpLoader,
}

impl DbgHelpSymbolEngine {
    pub fn new() -> Result<Self, DebugError> {
        let loader = DbgHelpLoader::new()?;
        Ok(Self {
            process_handle: 0,
            loader,
        })
    }

    fn check_bool(&self, result: BOOL, msg: &str) -> Result<(), DebugError> {
        if result == 0 {
            let code = unsafe { windows::Win32::Foundation::GetLastError().0 };
            Err(DebugError::Win32Error {
                code,
                message: msg.into(),
            })
        } else {
            Ok(())
        }
    }
}

impl crate::SymbolEngine for DbgHelpSymbolEngine {
    fn initialize(&mut self, process_id: u32) -> Result<(), DebugError> {
        // Open the process with query + vm_read access
        let handle = unsafe {
            windows::Win32::System::Threading::OpenProcess(
                windows::Win32::System::Threading::PROCESS_QUERY_INFORMATION
                    | windows::Win32::System::Threading::PROCESS_VM_READ,
                false,
                process_id,
            )
        };
        let handle = handle.map_err(|e| DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("OpenProcess failed for PID {process_id}"),
        })?;

        self.process_handle = handle.0;

        let result = unsafe { (self.loader.sym_initialize_w)(self.process_handle, std::ptr::null(), 1) };
        self.check_bool(result, "SymInitializeW failed")?;

        info!("DbgHelp initialized for PID {process_id}");
        Ok(())
    }

    fn resolve(&self, address: u64) -> Result<SymbolInfo, DebugError> {
        const MAX_NAME_LEN: usize = 512;
        let mut buffer = vec![0u8; std::mem::size_of::<SYMBOL_INFO>() + MAX_NAME_LEN];
        let sym = unsafe { &mut *(buffer.as_mut_ptr() as *mut SYMBOL_INFO) };
        sym.size_of_struct = std::mem::size_of::<SYMBOL_INFO>() as DWORD;
        sym.max_name_len = MAX_NAME_LEN as DWORD;

        let result = unsafe {
            (self.loader.sym_from_addr_w)(
                self.process_handle,
                address,
                std::ptr::null_mut(),
                sym,
            )
        };

        if result == 0 {
            return Err(DebugError::Internal(format!(
                "SymFromAddrW failed for 0x{address:x}"
            )));
        }

        let name = unsafe {
            let slice = std::slice::from_raw_parts(sym.name.as_ptr(), sym.name_len as usize);
            std::str::from_utf8_unchecked(slice)
        };

        let mut line_info = IMAGEHLP_LINE64 {
            size_of_struct: std::mem::size_of::<IMAGEHLP_LINE64>() as DWORD,
            key: std::ptr::null_mut(),
            line_number: 0,
            file_name: std::ptr::null_mut(),
            address: 0,
        };
        let mut displacement: DWORD = 0;
        let line_result = unsafe {
            (self.loader.sym_get_line_from_addr_w64)(
                self.process_handle,
                address,
                &mut displacement,
                &mut line_info,
            )
        };

        let (file, line) = if line_result != 0 && !line_info.file_name.is_null() {
            let file_str = unsafe { CStr::from_ptr(line_info.file_name as *const i8) }
                .to_string_lossy()
                .into_owned();
            (Some(file_str), Some(line_info.line_number))
        } else {
            (None, None)
        };

        Ok(SymbolInfo {
            name: crate::demangler::demangle(name),
            module: format!("0x{:x}", sym.mod_base),
            file,
            line,
            address: sym.address,
        })
    }

    fn walk_stack(&self, _thread_id: u32) -> Result<Vec<StackFrame>, DebugError> {
        warn!("walk_stack requires thread context (CONTEXT) which is not yet exposed cross-crate");
        Ok(vec![])
    }

    fn load_module(&mut self, base: u64, path: &str) -> Result<(), DebugError> {
        info!("Loading symbols for module at 0x{base:x}: {path}");
        let wide_path: Vec<u16> = std::ffi::OsStr::new(path)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let result = unsafe {
            (self.loader.sym_load_module_ex_w)(
                self.process_handle,
                0, // hFile
                wide_path.as_ptr(),
                std::ptr::null(), // ModuleName (optional)
                base,
                0, // SizeOfDll (0 = auto-detect from PE headers)
                std::ptr::null_mut(), // Data
                0, // Flags
            )
        };
        if result == 0 {
            let code = unsafe { windows::Win32::Foundation::GetLastError().0 };
            if code != 0 {
                return Err(DebugError::Win32Error {
                    code,
                    message: format!("SymLoadModuleExW failed for {path}"),
                });
            }
        }
        info!("Symbols loaded for module at 0x{base:x}");
        Ok(())
    }

    fn unload_module(&mut self, base: u64) -> Result<(), DebugError> {
        info!("Unloading symbols for module at 0x{base:x}");
        let result = unsafe {
            (self.loader.sym_unload_module64)(self.process_handle, base)
        };
        self.check_bool(result, "SymUnloadModule64 failed")?;
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), DebugError> {
        if self.process_handle != 0 {
            let result = unsafe { (self.loader.sym_cleanup)(self.process_handle) };
            let _ = self.check_bool(result, "SymCleanup failed");
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(windows::Win32::Foundation::HANDLE(self.process_handle));
            }
            self.process_handle = 0;
        }
        Ok(())
    }
}

impl Drop for DbgHelpSymbolEngine {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
