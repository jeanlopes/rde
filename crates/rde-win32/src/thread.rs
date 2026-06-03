//! Thread context and suspension wrappers.

use rde_core::{DebugError, ProcessHandle, RawHandle, RegisterContext, ThreadId};
use tracing::instrument;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::{
    GetThreadContext, SetThreadContext, CONTEXT, CONTEXT_ALL_AMD64,
};
use windows::Win32::System::Threading::{OpenThread, ResumeThread, SuspendThread, THREAD_ALL_ACCESS};

/// Wrapper that guarantees 16-byte alignment for CONTEXT.
///
/// The `windows` crate's `CONTEXT` struct is `#[repr(C)]` without explicit alignment.
/// On x64, `GetThreadContext`/`SetThreadContext` require `CONTEXT` to be 16-byte
/// aligned; otherwise they may fail with ERROR_NOACCESS or silently corrupt fields
/// (e.g. truncating RIP to 32 bits).
#[repr(C, align(16))]
#[derive(Default)]
struct AlignedContext(CONTEXT);

impl AlignedContext {
    fn as_mut_ptr(&mut self) -> *mut CONTEXT {
        &mut self.0
    }
    fn as_ptr(&self) -> *const CONTEXT {
        &self.0
    }
}

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
    let mut aligned = AlignedContext::default();
    aligned.0.ContextFlags = CONTEXT_ALL_AMD64;
    tracing::debug!("get_context: aligned ptr = {:p}, align = {}, sizeof(CONTEXT) = {}, sizeof(AlignedContext) = {}, CONTEXT_ALL_AMD64 = 0x{:x}", &mut aligned, std::mem::align_of::<AlignedContext>(), std::mem::size_of::<CONTEXT>(), std::mem::size_of::<AlignedContext>(), CONTEXT_ALL_AMD64.0);

    // SAFETY: h is a valid thread handle; aligned is 16-byte aligned as required by x64 CONTEXT.
    let result = unsafe { GetThreadContext(h, aligned.as_mut_ptr()) };
    if let Err(e) = result {
        let _ = unsafe { CloseHandle(h) };
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("GetThreadContext failed for TID {thread_id}"),
        });
    }
    let _ = unsafe { CloseHandle(h) };
    let regs = context_to_regs(&aligned.0);
    tracing::debug!("get_context: returned Rip = 0x{:x}, Rflags = 0x{:x}", regs.rip, regs.rflags);
    Ok(regs)
}

/// Set thread context (registers).
#[instrument]
pub fn set_context(_handle: &ProcessHandle, thread_id: ThreadId, ctx: &RegisterContext) -> Result<(), DebugError> {
    let h = open_thread(thread_id)?;
    let mut aligned = AlignedContext::default();
    aligned.0 = regs_to_context(ctx);
    let rip = aligned.0.Rip;
    tracing::debug!("set_context: aligned ptr = {:p}, align = {}, Rip = 0x{:x}", &mut aligned, std::mem::align_of::<AlignedContext>(), rip);

    // CRITICAL FIX: We MUST read the current context first before writing.
    // The `windows` crate's `CONTEXT` struct lacks `#[repr(align(16))]`, but
    // more importantly, if we create a fresh `CONTEXT` (e.g. `SegCs = 0`),
    // `SetThreadContext` on x64 may misinterpret it as a WOW64 context and
    // truncate 64-bit registers (e.g. `Rip` upper 32 bits lost).
    // By reading first, we preserve `SegCs` (0x33) and all other fields.
    // See: https://stackoverflow.com/questions/48020269
    aligned.0.ContextFlags = CONTEXT_ALL_AMD64;
    let result = unsafe { GetThreadContext(h, aligned.as_mut_ptr()) };
    if let Err(e) = result {
        let _ = unsafe { CloseHandle(h) };
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("GetThreadContext (read-before-write) failed for TID {thread_id}"),
        });
    }
    // Apply only the register changes from the caller
    apply_regs_to_context(&mut aligned.0, ctx);

    // SAFETY: h is a valid thread handle; aligned is 16-byte aligned as required by x64 CONTEXT.
    let result = unsafe { SetThreadContext(h, aligned.as_ptr()) };
    if let Err(e) = result {
        let _ = unsafe { CloseHandle(h) };
        return Err(DebugError::Win32Error {
            code: e.code().0 as u32,
            message: format!("SetThreadContext failed for TID {thread_id}"),
        });
    }
    let _ = unsafe { CloseHandle(h) };
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
    apply_regs_to_context(&mut ctx, regs);
    ctx
}

/// Apply `RegisterContext` values to an existing `CONTEXT` struct.
///
/// This is used in `set_context` after reading the current context,
/// so that segment registers (especially `SegCs`) and other fields
/// are preserved.
fn apply_regs_to_context(ctx: &mut CONTEXT, regs: &RegisterContext) {
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
}

const _: () = {
    assert!(std::mem::size_of::<CONTEXT>() == 1232, "CONTEXT size mismatch");
    // NOTE: `windows` crate v0.56.0 defines x86_64 `CONTEXT` without
    // `#[repr(align(16))]`, so `align_of::<CONTEXT>()` is 8, not 16.
    // We wrap it in `AlignedContext` which IS 16-byte aligned, and
    // we read-before-write in `set_context` to preserve `SegCs`.
    // Do NOT assert alignment here or compilation will fail.
};
