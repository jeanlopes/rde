//! Pretty printer for `Option<T>`.

use crate::{FormatBudget, MemoryReader, PrettyPrinter, PrettyValue};
use rde_core::DebugError;

pub struct OptionPrinter;

impl PrettyPrinter for OptionPrinter {
    fn format(
        &self,
        reader: &dyn MemoryReader,
        address: usize,
        budget: &mut FormatBudget,
    ) -> Result<PrettyValue, DebugError> {
        if budget.is_exhausted() {
            return Ok(PrettyValue::Truncated);
        }

        // Read discriminant (first byte for most Option<T> layouts on x86-64).
        let disc = reader.read_bytes(address, 1)?[0];

        if disc == 0 {
            Ok(PrettyValue::Enum {
                name: "None".to_string(),
                payload: None,
            })
        } else {
            // Payload starts after discriminant padding; for simple types
            // we read a placeholder. Real implementation needs type info.
            Ok(PrettyValue::Enum {
                name: "Some".to_string(),
                payload: Some(Box::new(PrettyValue::Raw(format!(
                    "(value at 0x{:x})",
                    address + 1
                )))),
            })
        }
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
            Ok(self.data[address..end].to_vec())
        }
    }

    #[test]
    fn test_option_none() {
        let reader = MockReader { data: vec![0u8; 8] };
        let printer = OptionPrinter;
        let mut budget = FormatBudget::default();
        let value = printer.format(&reader, 0, &mut budget).unwrap();
        assert_eq!(
            value,
            PrettyValue::Enum {
                name: "None".to_string(),
                payload: None,
            }
        );
    }

    #[test]
    fn test_option_some() {
        let reader = MockReader { data: vec![1u8; 8] };
        let printer = OptionPrinter;
        let mut budget = FormatBudget::default();
        let value = printer.format(&reader, 0, &mut budget).unwrap();
        assert_eq!(
            value,
            PrettyValue::Enum {
                name: "Some".to_string(),
                payload: Some(Box::new(PrettyValue::Raw("(value at 0x1)".to_string()))),
            }
        );
    }
}
