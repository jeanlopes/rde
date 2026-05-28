//! Integration test: pretty-print a mocked memory region.

use rde_core::memory::MemoryReader;
use rde_core::DebugError;
use rde_pretty_print::{built_in_registry, FormatBudget, PrettyValue};

struct MockMemory {
    mem: Vec<u8>,
}

impl MemoryReader for MockMemory {
    fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>, DebugError> {
        let end = (address + len).min(self.mem.len());
        if address >= self.mem.len() {
            return Ok(vec![0u8; len]);
        }
        Ok(self.mem[address..end].to_vec())
    }
}

#[test]
fn test_pretty_print_vec_end_to_end() {
    // Layout: ptr=0x20, len=3, cap=4
    let mut mem = vec![0u8; 64];
    mem[0..8].copy_from_slice(&32usize.to_le_bytes());
    mem[8..16].copy_from_slice(&3usize.to_le_bytes());
    mem[32..35].copy_from_slice(&[10, 20, 30]);

    let reader = MockMemory { mem };
    let registry = built_in_registry();
    let mut budget = FormatBudget::default();

    let printer = registry.lookup("std::vec::Vec").expect("Vec printer registered");
    let value = printer.format(&reader, 0, &mut budget).unwrap();

    assert_eq!(
        value,
        PrettyValue::Sequence(vec![
            PrettyValue::Scalar("10".to_string()),
            PrettyValue::Scalar("20".to_string()),
            PrettyValue::Scalar("30".to_string()),
        ])
    );
}
