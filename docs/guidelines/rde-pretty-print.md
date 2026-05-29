# rde-pretty-print

Human-readable formatting for Rust standard types from debuggee memory.

---

## Overview

`rde-pretty-print` reads raw bytes from a debuggee's memory and formats them into human-readable representations of Rust standard library types.

## Supported Types

| Type | Status | Example Output |
|------|--------|---------------|
| `Option<T>` | ✅ Full | `Some(42)`, `None` |
| `Vec<T>` | ✅ Full | `[10, 20, 30]` |
| `String` | ✅ Full | `"hello"` |
| `HashMap<K,V>` | ⚠️ Summary | `HashMap { ... }` |
| `BTreeMap<K,V>` | ⚠️ Summary | `BTreeMap { ... }` |
| `Result<T,E>` | ⚠️ Summary | `Result { ... }` |

## Core Types

### PrettyValue

```rust
pub enum PrettyValue {
    Scalar(String),
    Enum { name: String, payload: Option<Box<PrettyValue>> },
    Sequence(Vec<PrettyValue>),
    Map(Vec<(PrettyValue, PrettyValue)>),
    Raw(String),
    Truncated,
}
```

### FormatBudget

Prevents infinite recursion and excessive memory reads:

```rust
pub struct FormatBudget {
    pub max_depth: u32,      // Default: 5
    pub max_elements: u32,   // Default: 100
    pub max_bytes: usize,    // Default: 4096
}
```

```rust
use rde_pretty_print::FormatBudget;

// Default budget
let mut budget = FormatBudget::default();

// Custom budget
let mut budget = FormatBudget::new(10, 500, 8192);
```

### MemoryReader

```rust
use rde_core::memory::MemoryReader;
use rde_core::DebugError;

impl MemoryReader for MyReader {
    fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>, DebugError> {
        // Read from target process
    }
}
```

### PrettyPrinter Trait

```rust
pub trait PrettyPrinter: Send + Sync {
    fn format(
        &self,
        reader: &dyn MemoryReader,
        address: usize,
        budget: &mut FormatBudget,
    ) -> Result<PrettyValue, DebugError>;
}
```

## Built-in Printers

### Using the Registry

```rust
use rde_pretty_print::built_in_registry;

let registry = built_in_registry();
let printer = registry.lookup("std::vec::Vec").unwrap();
```

### Registered Type Names

| Type | Lookup Key |
|------|-----------|
| `Option<T>` | `std::option::Option`, `core::option::Option` |
| `Vec<T>` | `std::vec::Vec`, `core::vec::Vec`, `alloc::vec::Vec` |
| `String` | `std::string::String`, `core::string::String`, `alloc::string::String` |
| `HashMap<K,V>` | `std::collections::HashMap`, `std::collections::hash::map::HashMap` |
| `Result<T,E>` | `std::result::Result`, `core::result::Result` |

## Custom Printers

### Implementing PrettyPrinter

```rust
use rde_pretty_print::{PrettyPrinter, PrettyValue, FormatBudget, MemoryReader};
use rde_core::DebugError;

struct MyStructPrinter;

impl PrettyPrinter for MyStructPrinter {
    fn format(
        &self,
        reader: &dyn MemoryReader,
        address: usize,
        budget: &mut FormatBudget,
    ) -> Result<PrettyValue, DebugError> {
        if budget.is_exhausted() {
            return Ok(PrettyValue::Truncated);
        }

        // Read fields from memory
        let field1 = reader.read_bytes(address, 4)?;
        let value = u32::from_le_bytes(field1.try_into().unwrap());

        Ok(PrettyValue::Scalar(format!("MyStruct {{ field1: {} }}", value)))
    }
}
```

### Registering Custom Printers

```rust
use rde_pretty_print::PrinterRegistry;

let mut registry = PrinterRegistry::new();
registry.register("my_crate::MyStruct", Box::new(MyStructPrinter));
```

## Printer Implementations

### OptionPrinter

Reads the first byte as discriminant:
- `0` → `None`
- `1` → `Some(...)`

### VecPrinter

Reads the `Vec<T>` triple `(ptr, len, cap)` at the address:
- `ptr`: usize at offset 0
- `len`: usize at offset 8
- `cap`: usize at offset 16

Iterates up to `min(len, budget.max_elements)`.

### StringPrinter

Reads the `String` as a `Vec<u8>` wrapper and decodes UTF-8.

### HashMapPrinter

MVP: Returns summary only (`HashMap { ... }`). Full deep inspection requires stable `hashbrown` layout knowledge.

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rde_core::DebugError;

    struct MockReader {
        data: Vec<u8>,
    }

    impl MemoryReader for MockReader {
        fn read_bytes(&self, address: usize, len: usize) -> Result<Vec<u8>, DebugError> {
            let end = (address + len).min(self.data.len());
            if address >= self.data.len() {
                return Ok(vec![0u8; len]);
            }
            Ok(self.data[address..end].to_vec())
        }
    }

    #[test]
    fn test_vec_pretty_print() {
        let mut mem = vec![0u8; 64];
        mem[0..8].copy_from_slice(&32usize.to_le_bytes());
        mem[8..16].copy_from_slice(&3usize.to_le_bytes());
        mem[32..35].copy_from_slice(&[10, 20, 30]);

        let reader = MockReader { mem };
        let registry = built_in_registry();
        let mut budget = FormatBudget::default();
        let printer = registry.lookup("std::vec::Vec").unwrap();
        let value = printer.format(&reader, 0, &mut budget).unwrap();

        // value == PrettyValue::Sequence([Scalar("10"), Scalar("20"), Scalar("30")])
    }
}
```

## Performance

Benchmarks are in `crates/rde-pretty-print/tests/perf_bench.rs`.

| Benchmark | Target | Status |
|-----------|--------|--------|
| `Vec<T>` with 100 elements | < 1s | ✅ Passing |
| Nested structures | < 1s | ✅ Passing |
