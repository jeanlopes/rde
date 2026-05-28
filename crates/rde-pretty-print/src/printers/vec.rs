//! Pretty printer for `Vec<T>`.

use crate::{FormatBudget, MemoryReader, PrettyPrinter, PrettyValue};
use rde_core::DebugError;

pub struct VecPrinter;

impl PrettyPrinter for VecPrinter {
    fn format(
        &self,
        reader: &dyn MemoryReader,
        address: usize,
        budget: &mut FormatBudget,
    ) -> Result<PrettyValue, DebugError> {
        if budget.is_exhausted() {
            return Ok(PrettyValue::Truncated);
        }

        // Vec<T> layout on x86-64: (ptr: usize, len: usize, cap: usize)
        let ptr_bytes = reader.read_bytes(address, 8)?;
        let len_bytes = reader.read_bytes(address + 8, 8)?;

        let ptr = usize::from_le_bytes(ptr_bytes.try_into().unwrap());
        let len = usize::from_le_bytes(len_bytes.try_into().unwrap());

        let limit = budget.take_elements(len as u32) as usize;
        let mut elements = Vec::with_capacity(limit.min(100));

        for i in 0..limit {
            if budget.is_exhausted() {
                elements.push(PrettyValue::Truncated);
                break;
            }
            // Read a placeholder byte per element; real implementation
            // needs element type info from PDB.
            let elem_addr = ptr + i; // simplified: assumes byte-size elements
            let elem = reader.read_bytes(elem_addr, 1)?[0];
            elements.push(PrettyValue::Scalar(format!("{}", elem)));
        }

        if len > limit {
            elements.push(PrettyValue::Raw(format!("... and {} more", len - limit)));
        }

        Ok(PrettyValue::Sequence(elements))
    }
}

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
    fn test_vec_empty() {
        let reader = MockReader { data: vec![0u8; 24] };
        let printer = VecPrinter;
        let mut budget = FormatBudget::default();
        let value = printer.format(&reader, 0, &mut budget).unwrap();
        assert_eq!(value, PrettyValue::Sequence(vec![]));
    }

    #[test]
    fn test_vec_three_elements() {
        // ptr=0x20 (32), len=3, cap=4
        let mut data = vec![0u8; 64];
        data[0..8].copy_from_slice(&32usize.to_le_bytes());
        data[8..16].copy_from_slice(&3usize.to_le_bytes());
        data[32..35].copy_from_slice(&[10, 20, 30]);
        let reader = MockReader { data };
        let printer = VecPrinter;
        let mut budget = FormatBudget::default();
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
}
