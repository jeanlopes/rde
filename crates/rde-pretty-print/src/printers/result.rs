//! Pretty printer for `Result<T, E>`.

use crate::{FormatBudget, MemoryReader, PrettyPrinter, PrettyValue};
use rde_core::DebugError;

pub struct ResultPrinter;

impl PrettyPrinter for ResultPrinter {
    fn format(
        &self,
        _reader: &dyn MemoryReader,
        address: usize,
        budget: &mut FormatBudget,
    ) -> Result<PrettyValue, DebugError> {
        if budget.is_exhausted() {
            return Ok(PrettyValue::Truncated);
        }

        // Result<T,E> has the same layout as Option<T>: discriminant + payload.
        // Real implementation needs MemoryReader and type info.
        Ok(PrettyValue::Raw(format!(
            "Result {{ ... }} (at 0x{:x})",
            address
        )))
    }
}
