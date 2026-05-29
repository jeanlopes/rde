//! Pretty printer for `String`.

use crate::{FormatBudget, MemoryReader, PrettyPrinter, PrettyValue};
use rde_core::DebugError;

pub struct StringPrinter;

impl PrettyPrinter for StringPrinter {
    fn format(
        &self,
        reader: &dyn MemoryReader,
        address: usize,
        budget: &mut FormatBudget,
    ) -> Result<PrettyValue, DebugError> {
        if budget.is_exhausted() {
            return Ok(PrettyValue::Truncated);
        }

        // String is a wrapper around Vec<u8>: (ptr, len, cap)
        let ptr_bytes = reader.read_bytes(address, 8)?;
        let len_bytes = reader.read_bytes(address + 8, 8)?;

        let ptr = usize::from_le_bytes(ptr_bytes.try_into().unwrap());
        let len = usize::from_le_bytes(len_bytes.try_into().unwrap());

        let to_read = budget.take_bytes(len);
        let bytes = reader.read_bytes(ptr, to_read)?;

        let mut text = String::from_utf8_lossy(&bytes).to_string();
        if len > to_read {
            text.push_str("...");
        }

        Ok(PrettyValue::Scalar(format!("\"{}\"", text)))
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
    fn test_string_hello() {
        let mut data = vec![0u8; 64];
        let text = b"hello";
        let ptr = 32usize;
        data[0..8].copy_from_slice(&ptr.to_le_bytes());
        data[8..16].copy_from_slice(&text.len().to_le_bytes());
        data[ptr..ptr + text.len()].copy_from_slice(text);
        let reader = MockReader { data };
        let printer = StringPrinter;
        let mut budget = FormatBudget::default();
        let value = printer.format(&reader, 0, &mut budget).unwrap();
        assert_eq!(value, PrettyValue::Scalar("\"hello\"".to_string()));
    }
}
