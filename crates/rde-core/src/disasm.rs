//! Disassembly view — Capstone-based x86-64 instruction decoding.

use crate::DebugError;
use capstone::prelude::*;
use std::collections::HashSet;
use tracing::{info, instrument};

/// A single line of disassembly output.
#[derive(Debug, Clone)]
pub struct DisassemblyLine {
    pub address: u64,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
    pub is_current: bool,
    pub has_breakpoint: bool,
    pub original_bytes: Option<Vec<u8>>,
}

/// Collection of disassembly lines with context.
#[derive(Debug, Clone)]
pub struct DisassemblyView {
    pub lines: Vec<DisassemblyLine>,
    pub base_address: u64,
    pub rip: Option<u64>,
    pub truncated: bool,
}

/// Configuration for disassembly behavior.
#[derive(Debug, Clone)]
pub struct DisassemblyConfig {
    pub count: usize,
    pub auto_show: bool,
}

impl Default for DisassemblyConfig {
    fn default() -> Self {
        Self {
            count: 10,
            auto_show: false,
        }
    }
}

/// Disassembler using Capstone Engine.
pub struct Disassembler {
    cs: Capstone,
}

impl Disassembler {
    /// Create a new x86-64 disassembler in Intel syntax.
    pub fn new() -> Result<Self, DebugError> {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .build()
            .map_err(|e| DebugError::Internal(format!("Capstone init failed: {e}")))?;
        Ok(Self { cs })
    }

    /// Disassemble from the given byte buffer.
    #[instrument(skip(self, bytes), fields(address, count))]
    pub fn disassemble_at(
        &mut self,
        address: u64,
        count: usize,
        rip: Option<u64>,
        bytes: &[u8],
        active_breakpoints: &HashSet<u64>,
        breakpoints: &HashMap<u64, u8>,
    ) -> Result<DisassemblyView, DebugError> {
        info!(address = format!("0x{:x}", address), count, bytes_len = bytes.len(), "disassemble_at called");

        if bytes.is_empty() {
            return Err(DebugError::Internal(format!(
                "Cannot read memory at 0x{address:x}"
            )));
        }

        let insns = self
            .cs
            .disasm_all(bytes, address)
            .map_err(|e| DebugError::Internal(format!("Capstone disassembly failed: {e}")))?;

        let mut lines = Vec::new();
        for insn in insns.iter().take(count) {
            let addr = insn.address();
            let insn_bytes = insn.bytes().to_vec();
            let mnemonic = insn.mnemonic().unwrap_or("???").to_string();
            let operands = insn.op_str().unwrap_or("").to_string();
            let is_current = rip == Some(addr);
            let has_breakpoint = active_breakpoints.contains(&addr);
            let original_bytes = if has_breakpoint {
                breakpoints.get(&addr).map(|&b| vec![b])
            } else {
                None
            };

            lines.push(DisassemblyLine {
                address: addr,
                bytes: insn_bytes,
                mnemonic,
                operands,
                is_current,
                has_breakpoint,
                original_bytes,
            });
        }

        let truncated = lines.len() < count;
        info!(line_count = lines.len(), truncated, "disassembly complete");

        Ok(DisassemblyView {
            lines,
            base_address: address,
            rip,
            truncated,
        })
    }
}

impl DisassemblyView {
    /// Format the view as plain text for REPL output.
    pub fn format(&self) -> String {
        if self.lines.is_empty() {
            return "(no instructions)".to_string();
        }

        let mut lines = Vec::new();
        for line in &self.lines {
            let mut markers = String::new();
            if line.is_current {
                markers.push_str("=>");
            }
            if line.has_breakpoint {
                if !markers.is_empty() {
                    markers.push(' ');
                }
                markers.push_str("b+");
            }

            let marker_str = if markers.is_empty() {
                "    ".to_string()
            } else {
                format!("{:<4}", markers)
            };

            let bytes_hex: String = line
                .bytes
                .iter()
                .take(8)
                .map(|b| format!("{:02x} ", b))
                .collect::<String>()
                .trim_end()
                .to_string();
            let bytes_display = if line.bytes.len() > 8 {
                format!("{bytes_hex} ..")
            } else {
                bytes_hex
            };

            let mut comment = String::new();
            if line.has_breakpoint {
                if let Some(ref orig) = line.original_bytes {
                    if let Ok(view) = Disassembler::new() {
                        if let Ok(insns) = view.cs.disasm_all(orig, line.address) {
                            if let Some(insn) = insns.iter().next() {
                                let mnem = insn.mnemonic().unwrap_or("???");
                                let ops = insn.op_str().unwrap_or("");
                                if ops.is_empty() {
                                    comment = format!(" ; was: {mnem}");
                                } else {
                                    comment = format!(" ; was: {mnem} {ops}");
                                }
                            }
                        }
                    }
                }
            }

            lines.push(format!(
                "{} 0x{:016x}: {:<22} {:<8} {}{}",
                marker_str,
                line.address,
                bytes_display,
                line.mnemonic,
                line.operands,
                comment
            ));
        }

        if self.truncated {
            lines.push("(truncated)".to_string());
        }

        lines.join("\n")
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_nop() {
        let mut disasm = Disassembler::new().unwrap();
        let bytes = vec![0x90, 0x90, 0x90]; // 3x nop
        let active = HashSet::new();
        let bp_bytes = HashMap::new();
        let view = disasm.disassemble_at(0x1000, 10, None, &bytes, &active, &bp_bytes).unwrap();
        assert_eq!(view.lines.len(), 3);
        assert_eq!(view.lines[0].mnemonic, "nop");
        assert!(!view.lines[0].is_current);
        assert!(!view.lines[0].has_breakpoint);
    }

    #[test]
    fn test_disassemble_with_rip_marker() {
        let mut disasm = Disassembler::new().unwrap();
        let bytes = vec![0x90, 0x90, 0x90];
        let active = HashSet::new();
        let bp_bytes = HashMap::new();
        let view = disasm.disassemble_at(0x1000, 10, Some(0x1001), &bytes, &active, &bp_bytes).unwrap();
        assert!(!view.lines[0].is_current); // 0x1000
        assert!(view.lines[1].is_current);  // 0x1001
        assert!(!view.lines[2].is_current); // 0x1002
    }

    #[test]
    fn test_disassemble_with_breakpoint() {
        let mut disasm = Disassembler::new().unwrap();
        let bytes = vec![0x90, 0x90, 0x90];
        let mut active = HashSet::new();
        active.insert(0x1001);
        let mut bp_bytes = HashMap::new();
        bp_bytes.insert(0x1001, 0x90);
        let view = disasm.disassemble_at(0x1000, 10, None, &bytes, &active, &bp_bytes).unwrap();
        assert!(!view.lines[0].has_breakpoint);
        assert!(view.lines[1].has_breakpoint);
        assert!(!view.lines[2].has_breakpoint);
    }

    #[test]
    fn test_format_output() {
        let mut disasm = Disassembler::new().unwrap();
        let bytes = vec![0x48, 0x89, 0x5c, 0x24, 0x08]; // mov qword ptr [rsp+8], rbx
        let active = HashSet::new();
        let bp_bytes = HashMap::new();
        let view = disasm.disassemble_at(0x7ff6_1234_0000, 10, None, &bytes, &active, &bp_bytes).unwrap();
        let formatted = view.format();
        assert!(formatted.contains("mov"));
        assert!(formatted.contains("0x00007ff612340000"));
    }
}
