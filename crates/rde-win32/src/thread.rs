//! Thread context and suspension wrappers.

use rde_core::{DebugError, ProcessHandle, RawHandle, RegisterContext, ThreadId};
use tracing::instrument;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::{
    GetThreadContext, SetThreadContext, CONTEXT, CONTEXT_ALL_AMD64,
};
use windows::Win32::System::Threading::{OpenThread, ResumeThread, SuspendThread, THREAD_ALL_ACCESS};

/// Close a thread handle safely.
pub fn close_thread_handle(handle: RawHandle) {
    if handle.0 != 0 {
        // SAFETY: Closing a valid handle obtained from OpenThread or CreateThread debug event.
        let _ = unsafe { CloseHandle(HANDLE(handle.0 as isize)) };
    }
}

/// Open a thread handle by ID.
fn open_thread(thread_id: ThreadId) -> Result<HANDLE, DebugError> {
    // SAFETY: OpenThread accepts a valid thread ID. We request all access for debugging.
    // GitHub issue: TODO(rde#1)
    let handle = unsafe { OpenThread(THREAD_ALL_ACCESS, false, thread_id) };
    handle.map_err(|e| DebugError::Win32Error {
        code: e.code().0 as u32,
        message: format!("OpenThread failed for TID {thread_id}"),
    })
}

/// Get thread context (registers).
#[instrument]
pub fn get_context(_handle: &ProcessHandle, thread_id: ThreadId) -> Result<RegisterContext, DebugError> {
    let h = open_thread(thread_id)?;
    let mut ctx = CONTEXT::default();
    ctx.ContextFlags = CONTEXT_ALL_AMD64;

    // SAFETY: h is a valid thread handle; ctx is a properly initialized CONTEXT struct.
    // GitHub issue: TODO(rde#1)
    let result = unsafe { GetThreadContext(h, &mut ctx) };
    if let Err(e) = result {
        let _ = unsafe { CloseHandle(h) };
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("GetThreadContext failed for TID {thread_id}"),
        });
    }
    let _ = unsafe { CloseHandle(h) };

    Ok(context_to_regs(&ctx))
}

/// Set thread context (registers).
#[instrument]
pub fn set_context(_handle: &ProcessHandle, thread_id: ThreadId, ctx: &RegisterContext) -> Result<(), DebugError> {
    let h = open_thread(thread_id)?;
    let wctx = regs_to_context(ctx);

    // SAFETY: h is a valid thread handle; wctx is a properly initialized CONTEXT struct.
    // GitHub issue: TODO(rde#1)
    let result = unsafe { SetThreadContext(h, &wctx) };
    let _ = unsafe { CloseHandle(h) };

    if let Err(e) = result {
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("SetThreadContext failed for TID {thread_id}"),
        });
    }
    Ok(())
}

/// Suspend a thread.
#[instrument]
pub fn suspend(_handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError> {
    let h = open_thread(thread_id)?;
    // SAFETY: h is a valid thread handle.
    // GitHub issue: TODO(rde#1)
    let prev = unsafe { SuspendThread(h) };
    let _ = unsafe { CloseHandle(h) };
    if prev == u32::MAX {
        return Err(DebugError::Win32Error {
            code: unsafe { windows::Win32::Foundation::GetLastError().0 },
            message: format!("SuspendThread failed for TID {thread_id}"),
        });
    }
    Ok(())
}

/// Resume a thread.
#[instrument]
pub fn resume(_handle: &ProcessHandle, thread_id: ThreadId) -> Result<(), DebugError> {
    let h = open_thread(thread_id)?;
    // SAFETY: h is a valid thread handle.
    // GitHub issue: TODO(rde#1)
    let prev = unsafe { ResumeThread(h) };
    let _ = unsafe { CloseHandle(h) };
    if prev == u32::MAX {
        return Err(DebugError::Win32Error {
            code: unsafe { windows::Win32::Foundation::GetLastError().0 },
            message: format!("ResumeThread failed for TID {thread_id}"),
        });
    }
    Ok(())
}

fn context_to_regs(ctx: &CONTEXT) -> RegisterContext {
    RegisterContext {
        rax: ctx.Rax,
        rbx: ctx.Rbx,
        rcx: ctx.Rcx,
        rdx: ctx.Rdx,
        rsi: ctx.Rsi,
        rdi: ctx.Rdi,
        rbp: ctx.Rbp,
        rsp: ctx.Rsp,
        rip: ctx.Rip,
        r8: ctx.R8,
        r9: ctx.R9,
        r10: ctx.R10,
        r11: ctx.R11,
        r12: ctx.R12,
        r13: ctx.R13,
        r14: ctx.R14,
        r15: ctx.R15,
        rflags: ctx.EFlags as u64,
    }
}

fn regs_to_context(regs: &RegisterContext) -> CONTEXT {
    let mut ctx = CONTEXT::default();
    ctx.ContextFlags = CONTEXT_ALL_AMD64;
    ctx.Rax = regs.rax;
    ctx.Rbx = regs.rbx;
    ctx.Rcx = regs.rcx;
    ctx.Rdx = regs.rdx;
    ctx.Rsi = regs.rsi;
    ctx.Rdi = regs.rdi;
    ctx.Rbp = regs.rbp;
    ctx.Rsp = regs.rsp;
    ctx.Rip = regs.rip;
    ctx.R8 = regs.r8;
    ctx.R9 = regs.r9;
    ctx.R10 = regs.r10;
    ctx.R11 = regs.r11;
    ctx.R12 = regs.r12;
    ctx.R13 = regs.r13;
    ctx.R14 = regs.r14;
    ctx.R15 = regs.r15;
    ctx.EFlags = regs.rflags as u32;
    ctx
}
