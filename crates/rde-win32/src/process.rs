//! Process launching and attaching.
//!
//! CRITICAL INVARIANT — THREAD AFFINITY:
//! `CreateProcessW` with `DEBUG_PROCESS` must be called on the same thread
//! that will later call `WaitForDebugEventEx`. Violation produces silent
//! failures or ERROR_INVALID_HANDLE.
//!
//! RDE enforcement: `launch_sync` is called from inside the debug loop thread.

use rde_core::{DebugError, ProcessHandle, RawHandle};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use tracing::instrument;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE, OPEN_EXISTING};
use windows::Win32::System::Diagnostics::Debug::DebugActiveProcess;
use windows::Win32::System::Threading::{
    CreateProcessW, GetProcessId, STARTUPINFOW, STARTF_USESTDHANDLES,
};

const DEBUG_PROCESS: u32 = 0x00000001;
const DEBUG_ONLY_THIS_PROCESS: u32 = 0x00000002;

/// Launch a process under the debugger.
///
/// INVARIANT: This function must be called from the thread that will run
/// `WaitForDebugEventEx` (the debug loop thread).
#[instrument]
pub fn launch(path: &Path, args: &[String]) -> Result<ProcessHandle, DebugError> {
    let mut cmdline = build_command_line(path, args);

    // Open NUL device to suppress debuggee stdout/stderr in TUI mode.
    // SAFETY: CreateFileW with a literal path is safe; we check the result.
    let nul_wide: Vec<u16> = std::ffi::OsStr::new("NUL")
        .encode_wide()
        .chain(Some(0))
        .collect();
    let nul_handle = unsafe {
        CreateFileW(
            windows::core::PCWSTR(nul_wide.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE(0),
        )
    };
    let nul_handle = match nul_handle {
        Ok(h) => h,
        Err(_) => {
            return Err(DebugError::Win32Error {
                code: unsafe { windows::Win32::Foundation::GetLastError().0 },
                message: "Failed to open NUL device for debuggee redirection".into(),
            });
        }
    };

    let mut startup_info = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        dwFlags: STARTF_USESTDHANDLES,
        hStdOutput: nul_handle,
        hStdError: nul_handle,
        ..Default::default()
    };
    let mut process_info = unsafe { std::mem::zeroed() };

    // SAFETY: We pass valid wide strings and out-pointers. CreateProcessW is a standard
    // Windows API call. The returned handles are owned and closed on error paths.
    // bInheritHandles MUST be true so the child can use the NUL handles.
    // GitHub issue: TODO(rde#1) — track unsafe Win32 API usage
    let result = unsafe {
        CreateProcessW(
            None, // Application name is included in command line
            windows::core::PWSTR(cmdline.as_mut_ptr()),
            None, // Process security attributes
            None, // Thread security attributes
            true, // Inherit handles so child gets NUL stdout/stderr
            windows::Win32::System::Threading::PROCESS_CREATION_FLAGS(DEBUG_PROCESS | DEBUG_ONLY_THIS_PROCESS),
            None, // Environment
            None, // Current directory
            &mut startup_info,
            &mut process_info,
        )
    };

    // Close our copy of the NUL handle; the child inherited its own reference.
    let _ = unsafe { CloseHandle(nul_handle) };

    if let Err(e) = result {
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("CreateProcessW failed for {}", path.display()),
        });
    }

    // Close thread handle — we track threads via debug events
    // SAFETY: process_info.hThread is a valid handle returned by CreateProcessW.
    let _ = unsafe { CloseHandle(process_info.hThread) };

    let pid = unsafe { GetProcessId(process_info.hProcess) };

    Ok(ProcessHandle {
        process_id: pid,
        process_handle: RawHandle(process_info.hProcess.0 as usize),
    })
}

/// Attach to an existing process.
#[instrument]
pub fn attach(pid: u32) -> Result<ProcessHandle, DebugError> {
    // SAFETY: DebugActiveProcess accepts any valid PID. We validate the result.
    // GitHub issue: TODO(rde#1)
    let result = unsafe { DebugActiveProcess(pid) };
    if let Err(e) = result {
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("DebugActiveProcess failed for PID {pid}"),
        });
    }

    // We don't have a process handle yet; the debug loop will receive CREATE_PROCESS_DEBUG_EVENT
    Ok(ProcessHandle {
        process_id: pid,
        process_handle: RawHandle(0),
    })
}

fn build_command_line(path: &Path, args: &[String]) -> Vec<u16> {
    let mut cmd = vec![b'"' as u16];
    cmd.extend(path.as_os_str().encode_wide());
    cmd.push(b'"' as u16);
    for arg in args {
        cmd.push(b' ' as u16);
        cmd.push(b'"' as u16);
        cmd.extend(OsStr::new(arg).encode_wide());
        cmd.push(b'"' as u16);
    }
    cmd.push(0);
    cmd
}
