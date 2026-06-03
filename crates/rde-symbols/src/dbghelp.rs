//! DbgHelp.dll dynamic loading and symbol engine implementation.

use crate::SymbolEngine;
use libloading::{Library, Symbol};
use rde_core::{DebugError, StackFrame, SymbolInfo};
use std::ffi::{c_void, CStr};
use std::os::windows::ffi::OsStrExt;
use tracing::info;

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
type SymFromNameWFn = unsafe extern "system" fn(HANDLE, *const u16, *mut SYMBOL_INFO) -> BOOL;
type SymSetOptionsFn = unsafe extern "system" fn(DWORD) -> DWORD;
type SymEnumSymbolsWFn = unsafe extern "system" fn(
    HANDLE,
    DWORD64,
    *const u16,
    Option<unsafe extern "system" fn(*mut SYMBOL_INFO, DWORD64, *mut c_void) -> BOOL>,
    *mut c_void,
) -> BOOL;
type StackWalk64Fn = unsafe extern "system" fn(
    DWORD,
    HANDLE,
    HANDLE,
    *mut STACKFRAME64,
    *mut c_void, // CONTEXT
    Option<unsafe extern "system" fn(HANDLE, DWORD64, *mut c_void, DWORD, *mut DWORD) -> BOOL>,
    Option<unsafe extern "system" fn(HANDLE, DWORD64) -> *mut c_void>,
    Option<unsafe extern "system" fn(HANDLE, DWORD64) -> DWORD64>,
    Option<unsafe extern "system" fn(DWORD, *mut STACKFRAME64, *mut c_void, DWORD) -> BOOL>,
) -> BOOL;
type SymFunctionTableAccess64Fn = unsafe extern "system" fn(HANDLE, DWORD64) -> *mut c_void;
type SymGetModuleBase64Fn = unsafe extern "system" fn(HANDLE, DWORD64) -> DWORD64;

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
    sym_from_name_w: Symbol<'static, SymFromNameWFn>,
    #[allow(dead_code)]
    sym_set_options: Symbol<'static, SymSetOptionsFn>,
    sym_enum_symbols_w: Symbol<'static, SymEnumSymbolsWFn>,
    #[allow(dead_code)]
    stack_walk64: Symbol<'static, StackWalk64Fn>,
    sym_function_table_access64: Symbol<'static, SymFunctionTableAccess64Fn>,
    sym_get_module_base64: Symbol<'static, SymGetModuleBase64Fn>,
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
        let sym_function_table_access64 = unsafe {
            lib.get(b"SymFunctionTableAccess64\0")
                .map_err(|e| DebugError::Internal(format!("SymFunctionTableAccess64 not found: {e}")))?
        };
        let sym_get_module_base64 = unsafe {
            lib.get(b"SymGetModuleBase64\0")
                .map_err(|e| DebugError::Internal(format!("SymGetModuleBase64 not found: {e}")))?
        };
        let sym_load_module_ex_w = unsafe {
            lib.get(b"SymLoadModuleExW\0")
                .map_err(|e| DebugError::Internal(format!("SymLoadModuleExW not found: {e}")))?
        };
        let sym_unload_module64 = unsafe {
            lib.get(b"SymUnloadModule64\0")
                .map_err(|e| DebugError::Internal(format!("SymUnloadModule64 not found: {e}")))?
        };
        let sym_from_name_w = unsafe {
            lib.get(b"SymFromNameW\0")
                .map_err(|e| DebugError::Internal(format!("SymFromNameW not found: {e}")))?
        };
        let sym_set_options: Symbol<'static, SymSetOptionsFn> = unsafe {
            lib.get(b"SymSetOptions\0")
                .map_err(|e| DebugError::Internal(format!("SymSetOptions not found: {e}")))?
        };
        let sym_enum_symbols_w = unsafe {
            lib.get(b"SymEnumSymbolsW\0")
                .map_err(|e| DebugError::Internal(format!("SymEnumSymbolsW not found: {e}")))?
        };

        // SYMOPT_UNDNAME (0x0001): undecorate symbols
        // SYMOPT_LOAD_LINES (0x0010): include line number information
        // NOTE: SYMOPT_DEFERRED_LOADS is intentionally NOT set because SymEnumSymbolsW
        // does not trigger deferred loading — symbols must be loaded upfront for enumeration.
        // The local-only symbol path (set before initialize) prevents slow symbol-server access.
        unsafe { (*sym_set_options)(0x0001 | 0x0010) };

        Ok(Self {
            _lib: unsafe { Library::new("DbgHelp.dll").unwrap() }, // placeholder, not used
            sym_initialize_w,
            sym_cleanup,
            sym_from_addr_w,
            sym_get_line_from_addr_w64,
            sym_load_module_ex_w,
            sym_unload_module64,
            sym_from_name_w,
            sym_set_options,
            sym_enum_symbols_w,
            stack_walk64,
            sym_function_table_access64,
            sym_get_module_base64,
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
    symbol_path: Option<Vec<u16>>,
    main_module_base: Option<u64>,
    pdb_symbols: std::collections::HashMap<String, u64>,
}

impl DbgHelpSymbolEngine {
    pub fn new() -> Result<Self, DebugError> {
        let loader = DbgHelpLoader::new()?;
        Ok(Self {
            process_handle: 0,
            loader,
            symbol_path: None,
            main_module_base: None,
            pdb_symbols: std::collections::HashMap::new(),
        })
    }

    /// Restrict symbol loading to the given directory (no symbol-server network access).
    /// Must be called before `initialize`.
    pub fn set_symbol_path(&mut self, path: &str) {
        let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        self.symbol_path = Some(wide);
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

    fn open_process_handle(&mut self, process_id: u32) -> Result<(), DebugError> {
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
        Ok(())
    }
}

impl DbgHelpSymbolEngine {
    fn load_pdb_symbols(&mut self, exe_path: &std::path::Path, base_address: u64) {
        use std::collections::HashMap;
        use pdb::FallibleIterator;
        let mut symbols: HashMap<String, u64> = HashMap::new();

        // Derive PDB path from exe path (same directory, same name, .pdb extension)
        let pdb_path = if let Some(stem) = exe_path.file_stem() {
            let mut p = exe_path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            p.push(format!("{}.pdb", stem.to_string_lossy()));
            p
        } else {
            return;
        };

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut f = OpenOptions::new().create(true).append(true).open("C:/workspace/rde/sym_debug.log").unwrap();
            writeln!(f, "load_pdb_symbols: exe={} pdb={} exists={} base=0x{:x}", exe_path.display(), pdb_path.display(), pdb_path.exists(), base_address).unwrap();
            f.flush().unwrap();
        }

        if !pdb_path.exists() {
            tracing::warn!("PDB file not found at {}", pdb_path.display());
            return;
        }

        let file = match std::fs::File::open(&pdb_path) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("Failed to open PDB {}: {}", pdb_path.display(), e);
                return;
            }
        };
        let mmap = match unsafe { memmap2::Mmap::map(&file) } {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to mmap PDB {}: {}", pdb_path.display(), e);
                return;
            }
        };
        let mut pdb = match pdb::PDB::open(std::io::Cursor::new(&*mmap)) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to parse PDB {}: {:?}", pdb_path.display(), e);
                return;
            }
        };

        // Load section headers to convert section+offset to virtual address.
        let sections = match pdb.sections() {
            Ok(Some(s)) => s,
            _ => Vec::new(),
        };

        let resolve_va = |section: u16, offset: u32| -> u64 {
            if section == 0 {
                base_address.saturating_add(offset as u64)
            } else {
                let idx = (section - 1) as usize;
                if idx < sections.len() {
                    base_address.saturating_add(sections[idx].virtual_address as u64).saturating_add(offset as u64)
                } else {
                    base_address.saturating_add(offset as u64)
                }
            }
        };

        // Read global symbols
        let global_table = match pdb.global_symbols() {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Failed to read global symbols from PDB {}: {:?}", pdb_path.display(), e);
                return;
            }
        };
        let mut iter = global_table.iter();
        while let Ok(Some(symbol)) = iter.next() {
            match symbol.parse() {
                Ok(pdb::SymbolData::Public(p)) if p.function => {
                    let name = p.name.to_string();
                    let va = resolve_va(p.offset.section, p.offset.offset);
                    let demangled = Self::demangle_rust_symbol(&name);
                    symbols.insert(demangled.clone(), va);
                    symbols.insert(name.to_string(), va);
                }
                Ok(pdb::SymbolData::Procedure(proc)) => {
                    let name = proc.name.to_string();
                    let va = resolve_va(proc.offset.section, proc.offset.offset);
                    let demangled = Self::demangle_rust_symbol(&name);
                    symbols.insert(demangled.clone(), va);
                    symbols.insert(name.to_string(), va);
                }
                _ => {}
            }
        }

        // Read per-module symbols (often contains more detailed info)
        let dbi = match pdb.debug_information() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("Failed to read DBI from PDB {}: {:?}", pdb_path.display(), e);
                return;
            }
        };
        let mut modules = match dbi.modules() {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to get modules from PDB {}: {:?}", pdb_path.display(), e);
                return;
            }
        };
        while let Ok(Some(module)) = modules.next() {
            let module_info = match pdb.module_info(&module) {
                Ok(Some(mi)) => mi,
                _ => continue,
            };
            let mut proc_iter = match module_info.symbols() {
                Ok(iter) => iter,
                Err(_) => continue,
            };
            while let Ok(Some(symbol)) = proc_iter.next() {
                match symbol.parse() {
                    Ok(pdb::SymbolData::Public(p)) if p.function => {
                        let name = p.name.to_string();
                        let va = resolve_va(p.offset.section, p.offset.offset);
                        let demangled = Self::demangle_rust_symbol(&name);
                        symbols.insert(demangled.clone(), va);
                        symbols.insert(name.to_string(), va);
                    }
                    Ok(pdb::SymbolData::Procedure(proc)) => {
                        let name = proc.name.to_string();
                        let va = resolve_va(proc.offset.section, proc.offset.offset);
                        let demangled = Self::demangle_rust_symbol(&name);
                        symbols.insert(demangled.clone(), va);
                        symbols.insert(name.to_string(), va);
                    }
                    _ => {}
                }
            }
        }

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut f = OpenOptions::new().create(true).append(true).open("C:/workspace/rde/sym_debug.log").unwrap();
            writeln!(f, "load_pdb_symbols: loaded {} symbols from {}", symbols.len(), pdb_path.display()).unwrap();
            for (k, v) in symbols.iter().take(5) {
                writeln!(f, "  sym: {} = 0x{:x}", k, v).unwrap();
            }
            f.flush().unwrap();
        }
        tracing::info!("Loaded {} symbols from PDB {}", symbols.len(), pdb_path.display());
        println!("[SYMBOLS] Loaded {} symbols from PDB {}", symbols.len(), pdb_path.display());
        self.pdb_symbols = symbols;
    }

    fn demangle_rust_symbol(mangled: &str) -> String {
        rustc_demangle::demangle(mangled).to_string()
    }
}

impl DbgHelpSymbolEngine {
    /// Walk the stack using a given RIP/RBP pair (e.g. from GetThreadContext).
    pub fn walk_stack_from_context(&self, rip: u64, rbp: u64) -> Result<Vec<StackFrame>, DebugError> {
        use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

        if self.process_handle == 0 {
            return Err(DebugError::Internal("DbgHelp not initialized".into()));
        }

        println!("[WALKSTACK] walk_stack_from_context RIP=0x{:x} RBP=0x{:x}", rip, rbp);

        let mut frames = Vec::new();
        let mut rip = rip;
        let mut rbp = rbp;
        let h_process = windows::Win32::Foundation::HANDLE(self.process_handle as isize);

        for i in 0..64 {
            if rip == 0 {
                break;
            }
            if rip < 0x1000 {
                break;
            }
            // Skip symbol resolution for system addresses to avoid DbgHelp hangs
            let symbol = if rip < 0x7FFF00000000 {
                self.resolve(rip).ok()
            } else {
                None
            };
            println!("[WALKSTACK] frame {i}: pc=0x{:x} rbp=0x{:x} sym={:?}", rip, rbp, symbol.as_ref().map(|s| &s.name[..]));
            frames.push(StackFrame {
                frame_number: i,
                return_address: rip,
                frame_pointer: rbp,
                stack_pointer: 0,
                symbol,
            });

            // Read next RBP and return address from stack using RBP chain.
            if rbp == 0 {
                break;
            }
            let mut next_rbp: u64 = 0;
            let mut ret_addr: u64 = 0;
            let ok1 = unsafe {
                ReadProcessMemory(
                    h_process,
                    rbp as *const std::ffi::c_void,
                    &mut next_rbp as *mut _ as *mut std::ffi::c_void,
                    8,
                    None,
                )
            };
            let ok2 = unsafe {
                ReadProcessMemory(
                    h_process,
                    (rbp + 8) as *const std::ffi::c_void,
                    &mut ret_addr as *mut _ as *mut std::ffi::c_void,
                    8,
                    None,
                )
            };
            if ok1.is_err() || ok2.is_err() {
                println!("[WALKSTACK] ReadProcessMemory failed at rbp=0x{:x}", rbp);
                break;
            }
            // Sanity checks for RBP chain:
            // - next_rbp must be 8-byte aligned
            // - next_rbp should be in user-mode stack range (below 0x7FFF00000000)
            // - next_rbp should not decrease by too much ( heuristic: must be > rbp - 0x10000 )
            // - detect loops
            if next_rbp == 0 || next_rbp == rbp || (next_rbp & 7) != 0 || next_rbp > 0x7FFF00000000 || next_rbp < rbp.saturating_sub(0x10000) {
                break;
            }
            // Extra safety: if ret_addr looks invalid, stop
            if ret_addr < 0x1000 || ret_addr > 0x7FFF00000000 {
                break;
            }
            rbp = next_rbp;
            rip = ret_addr;
        }

        eprintln!("[WALKSTACK] total frames: {}", frames.len());
        Ok(frames)
    }
}

impl crate::SymbolEngine for DbgHelpSymbolEngine {
    fn initialize(&mut self, process_id: u32, exe_path: Option<&std::path::Path>, base_address: Option<u64>, _process_handle: Option<isize>) -> Result<(), DebugError> {
        // Always open a fresh handle with VM_READ — the debug handle from CreateProcessW
        // does not work reliably with SymLoadModuleExW when fInvadeProcess=FALSE.
        self.open_process_handle(process_id)?;
        let sym_path_ptr = self
            .symbol_path
            .as_deref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        // Use fInvadeProcess=FALSE because we are called very early in the process
        // lifetime (before CreateProcess is continued). fInvadeProcess=TRUE would
        // fail because the PEB is not fully initialized. We load the main module
        // explicitly with SymLoadModuleExW afterwards.
        let result = unsafe { (self.loader.sym_initialize_w)(self.process_handle, sym_path_ptr, 0) };
        if result == 0 {
            let code = unsafe { windows::Win32::Foundation::GetLastError().0 };
            return Err(DebugError::Win32Error {
                code,
                message: format!("SymInitializeW failed for PID {process_id}"),
            });
        }

        // Explicitly load the main executable module to ensure its PDB is found.
        if let (Some(path), Some(base)) = (exe_path, base_address) {
            // Strip \\?\ verbatim prefix if present — SymLoadModuleExW doesn't handle it well.
            let path_str = path.to_string_lossy();
            let clean_path = if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
                stripped
            } else {
                &path_str
            };
            let wide_path: Vec<u16> = std::ffi::OsString::from(clean_path)
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            let module_base = unsafe {
                (self.loader.sym_load_module_ex_w)(
                    self.process_handle,
                    0,
                    wide_path.as_ptr(),
                    std::ptr::null(),
                    base,
                    0,
                    std::ptr::null_mut(),
                    0,
                )
            };
            if module_base == 0 {
                let code = unsafe { windows::Win32::Foundation::GetLastError().0 };
                tracing::warn!("SymLoadModuleExW failed with error {code} for path '{}' (clean='{}')", path.display(), clean_path);
                println!("[SYMBOLS] SymLoadModuleExW failed: code={code} path={} clean={}", path.display(), clean_path);
            } else {
                tracing::info!("SymLoadModuleExW succeeded: base=0x{module_base:x} for path {}", path.display());
                println!("[SYMBOLS] SymLoadModuleExW succeeded: base=0x{module_base:x} path={}", path.display());
                self.main_module_base = Some(module_base);
            }
        }

        // Fallback: load symbols from the PDB file directly using the pdb crate.
        if let Some(path) = exe_path {
            self.load_pdb_symbols(path, base_address.unwrap_or(0));
        }

        tracing::debug!("DbgHelp initialized for PID {process_id}");
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

        if result != 0 {
            let name = unsafe {
                let raw_len = sym.name_len as usize;
                let slice = std::slice::from_raw_parts(sym.name.as_ptr(), raw_len);
                // SymFromAddrW may write UTF-16LE into the name field even though
                // we pass a SYMBOL_INFO (ANSI) struct. Detect and convert.
                if raw_len >= 2 && slice[1] == 0 && slice.chunks(2).all(|c| c.len() == 1 || c[1] == 0) {
                    // name_len is in WCHARs; read raw_len * 2 bytes.
                    let byte_len = raw_len * 2;
                    let byte_slice = std::slice::from_raw_parts(sym.name.as_ptr(), byte_len);
                    let u16_slice: Vec<u16> = byte_slice.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                    String::from_utf16_lossy(&u16_slice)
                } else {
                    std::str::from_utf8_unchecked(slice).to_string()
                }
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

            return Ok(SymbolInfo {
                name: crate::demangler::demangle(&name),
                module: format!("0x{:x}", sym.mod_base),
                file,
                line,
                address: sym.address,
            });
        }

        // Fallback: search in our PDB-parsed symbol table for closest address match.
        // Stack walk PCs may point inside a function, not exactly at its start.
        let mut best_name: Option<&str> = None;
        let mut best_addr: u64 = 0;
        for (name, &addr) in &self.pdb_symbols {
            if addr <= address && addr > best_addr {
                best_addr = addr;
                best_name = Some(name);
            }
        }
        if let Some(name) = best_name {
            // Only use if we're within a reasonable function body (say 64KB)
            if address - best_addr < 0x10000 {
                println!("[RESOLVE] fallback addr=0x{:x} best_addr=0x{:x} name='{}' name_bytes={:?}", address, best_addr, name, name.as_bytes());
                return Ok(SymbolInfo {
                    name: crate::demangler::demangle(name),
                    module: "pdb_fallback".into(),
                    file: None,
                    line: None,
                    address: best_addr,
                });
            }
        }

        Err(DebugError::Internal(format!(
            "SymFromAddrW failed for 0x{address:x}"
        )))
    }

    fn walk_stack(&self, thread_id: u32) -> Result<Vec<StackFrame>, DebugError> {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            OpenThread,
            THREAD_ALL_ACCESS,
        };
        use windows::Win32::System::Diagnostics::Debug::{GetThreadContext, CONTEXT, CONTEXT_ALL_AMD64};

        if self.process_handle == 0 {
            return Err(DebugError::Internal("DbgHelp not initialized".into()));
        }

        let h_thread = unsafe {
            OpenThread(THREAD_ALL_ACCESS, false, thread_id)
        };
        let h_thread = match h_thread {
            Ok(h) => h,
            Err(e) => {
                return Err(DebugError::Win32Error {
                    code: e.code().0 as u32,
                    message: format!("OpenThread failed for TID {thread_id}"),
                });
            }
        };

        let mut ctx = CONTEXT::default();
        ctx.ContextFlags = CONTEXT_ALL_AMD64;

        let ctx_ok = unsafe { GetThreadContext(h_thread, &mut ctx) };
        unsafe { let _ = CloseHandle(h_thread); };
        if let Err(e) = ctx_ok {
            return Err(DebugError::Win32Error {
                code: e.code().0 as u32,
                message: format!("GetThreadContext failed for TID {thread_id}"),
            });
        }

        self.walk_stack_from_context(ctx.Rip, ctx.Rbp)
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

    fn resolve_by_name(&self, name: &str) -> Result<Vec<u64>, DebugError> {
        if self.process_handle == 0 {
            return Err(DebugError::Internal("DbgHelp not initialized".into()));
        }

        let mut results: Vec<u64> = Vec::new();

        // Try SymFromNameW first with exact name and common Rust prefixes.
        let variants = vec![
            name.to_string(),
            format!("rust_app_example::{name}"),
            format!("rust_app_example::{name}::{{closure}}"),
        ];
        for variant in &variants {
            let name_wide: Vec<u16> = variant.encode_utf16().chain(Some(0)).collect();
            const MAX_NAME_LEN: usize = 512;
            let mut buffer = vec![0u8; std::mem::size_of::<SYMBOL_INFO>() + MAX_NAME_LEN];
            let sym = unsafe { &mut *(buffer.as_mut_ptr() as *mut SYMBOL_INFO) };
            sym.size_of_struct = std::mem::size_of::<SYMBOL_INFO>() as DWORD;
            sym.max_name_len = MAX_NAME_LEN as DWORD;

            let r = unsafe {
                (self.loader.sym_from_name_w)(
                    self.process_handle,
                    name_wide.as_ptr(),
                    sym,
                )
            };
            if r != 0 && sym.address != 0 {
                results.push(sym.address);
                break;
            }
        }

        // Fallback to wildcard enumeration if exact lookup failed.
        if results.is_empty() {
            let pattern = format!("*{}*", name);
            let pattern_wide: Vec<u16> = pattern.encode_utf16().chain(Some(0)).collect();
            let results_ptr = &mut results as *mut Vec<u64> as *mut c_void;

            unsafe extern "system" fn enum_callback(
                sym_info: *mut SYMBOL_INFO,
                _symbol_size: DWORD64,
                user_context: *mut c_void,
            ) -> BOOL {
                if sym_info.is_null() || user_context.is_null() {
                    return 1;
                }
                let results = &mut *(user_context as *mut Vec<u64>);
                let address = (*sym_info).address;
                if address != 0 {
                    results.push(address);
                }
                1
            }

            let _ok = unsafe {
                (self.loader.sym_enum_symbols_w)(
                    self.process_handle,
                    0,
                    pattern_wide.as_ptr(),
                    Some(enum_callback),
                    results_ptr,
                )
            };
        }

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut f = OpenOptions::new().create(true).append(true).open("C:/workspace/rde/sym_debug.log").unwrap();
            writeln!(f, "resolve_by_name: name='{}' pdb_symbols_count={}", name, self.pdb_symbols.len()).unwrap();
            f.flush().unwrap();
        }
        if results.is_empty() {
            // Fallback: search in our PDB-parsed symbol table.
            let demangled_name = Self::demangle_rust_symbol(name);
            let search_names: Vec<&str> = vec![
                name,
                &demangled_name,
            ];
            for search_name in &search_names {
                if let Some(&addr) = self.pdb_symbols.get(*search_name) {
                    results.push(addr);
                }
            }
            // Also try substring match for cases where the user types a short name.
            // Prefer exact suffix matches (e.g. "::insert") over generic substrings
            // to avoid matching "insert_left" when looking for "insert".
            if results.is_empty() {
                let lower_name = name.to_lowercase();
                let suffix = format!("::{}" , lower_name);
                for (sym_name, &addr) in &self.pdb_symbols {
                    let lower_sym = sym_name.to_lowercase();
                    if lower_sym.ends_with(&suffix) || lower_sym == lower_name {
                        results.push(addr);
                    }
                }
                // If no suffix/exact match, fall back to generic substring
                if results.is_empty() {
                    for (sym_name, &addr) in &self.pdb_symbols {
                        if sym_name.to_lowercase().contains(&lower_name) {
                            results.push(addr);
                        }
                    }
                }
            }
            {
                use std::fs::OpenOptions;
                use std::io::Write;
                let mut f = OpenOptions::new().create(true).append(true).open("C:/workspace/rde/sym_debug.log").unwrap();
                writeln!(f, "resolve_by_name: fallback results={:?}", results).unwrap();
                f.flush().unwrap();
            }
        }

        if results.is_empty() {
            Err(DebugError::Internal(format!(
                "Symbol '{}' not found in loaded modules",
                name
            )))
        } else {
            results.sort_unstable();
            results.dedup();
            Ok(results)
        }
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
