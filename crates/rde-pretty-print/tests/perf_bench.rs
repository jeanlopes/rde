//! Performance benchmark: pretty-print Vec with 100 elements.

use rde_core::memory::MemoryReader;
use rde_core::DebugError;
use rde_pretty_print::{built_in_registry, FormatBudget};
use std::time::Instant;

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
fn bench_vec_100_elements() {
    let mut mem = vec![0u8; 1024];
    let ptr = 128usize;
    let len = 100usize;
    mem[0..8].copy_from_slice(&ptr.to_le_bytes());
    mem[8..16].copy_from_slice(&len.to_le_bytes());
    for i in 0..len {
        mem[ptr + i] = (i % 256) as u8;
    }

    let reader = MockMemory { mem };
    let registry = built_in_registry();
    let mut budget = FormatBudget::default();
    let printer = registry.lookup("std::vec::Vec").unwrap();

    let start = Instant::now();
    let _value = printer.format(&reader, 0, &mut budget).unwrap();
    let elapsed = start.elapsed();

    println!("Vec<100> pretty-print: {:?}", elapsed);
    assert!(elapsed.as_secs_f64() < 1.0, "Pretty-print took too long: {:?}", elapsed);
}

#[test]
fn bench_nested_option_vec() {
    let mut mem = vec![0u8; 512];
    let ptr = 64usize;
    let len = 10usize;
    mem[0..8].copy_from_slice(&ptr.to_le_bytes());
    mem[8..16].copy_from_slice(&len.to_le_bytes());
    for i in 0..len {
        mem[ptr + i] = (i % 256) as u8;
    }

    let reader = MockMemory { mem };
    let registry = built_in_registry();
    let mut budget = FormatBudget::default();
    let printer = registry.lookup("std::vec::Vec").unwrap();

    let start = Instant::now();
    let _value = printer.format(&reader, 0, &mut budget).unwrap();
    let elapsed = start.elapsed();

    println!("Nested Vec<10> pretty-print: {:?}", elapsed);
    assert!(elapsed.as_secs_f64() < 1.0, "Pretty-print took too long: {:?}", elapsed);
}
