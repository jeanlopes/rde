//! Output formatters for debug state.

use rde_core::{RegisterContext, StackFrame};

/// Format register context as a table.
pub fn format_registers(ctx: &RegisterContext) -> String {
    format!(
        "RAX: {:016X}  RBX: {:016X}\n\
         RCX: {:016X}  RDX: {:016X}\n\
         RSI: {:016X}  RDI: {:016X}\n\
         RBP: {:016X}  RSP: {:016X}\n\
         RIP: {:016X}  RFLAGS: {:016X}\n\
         R8:  {:016X}  R9:  {:016X}\n\
         R10: {:016X}  R11: {:016X}\n\
         R12: {:016X}  R13: {:016X}\n\
         R14: {:016X}  R15: {:016X}",
        ctx.rax, ctx.rbx, ctx.rcx, ctx.rdx, ctx.rsi, ctx.rdi, ctx.rbp, ctx.rsp,
        ctx.rip, ctx.rflags, ctx.r8, ctx.r9, ctx.r10, ctx.r11, ctx.r12, ctx.r13,
        ctx.r14, ctx.r15
    )
}

/// Format a hex dump with ASCII sidebar.
pub fn format_hex_dump(address: u64, bytes: &[u8]) -> String {
    let mut lines = Vec::new();
    for chunk in bytes.chunks(16) {
        let hex: String = chunk
            .iter()
            .map(|b| format!("{:02X} ", b))
            .collect();
        let ascii: String = chunk
            .iter()
            .map(|b| if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' })
            .collect();
        lines.push(format!("0x{:016X}: {:48} |{ascii}|", address, hex));
    }
    lines.join("\n")
}

/// Format a stack trace.
pub fn format_backtrace(frames: &[StackFrame]) -> String {
    let mut lines = Vec::new();
    for frame in frames {
        let sym = frame
            .symbol
            .as_ref()
            .map(|s| format!("{} + 0x{:x}", s.name, s.address))
            .unwrap_or_else(|| format!("0x{:x}", frame.return_address));
        lines.push(format!("#{} {sym}", frame.frame_number));
    }
    lines.join("\n")
}
