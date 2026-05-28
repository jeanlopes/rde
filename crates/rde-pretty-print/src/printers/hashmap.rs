//! Pretty printer for `HashMap<K, V>`.

use crate::{FormatBudget, MemoryReader, PrettyPrinter, PrettyValue};
use rde_core::DebugError;

pub struct HashMapPrinter;

impl PrettyPrinter for HashMapPrinter {
    fn format(
        &self,
        _reader: &dyn MemoryReader,
        _address: usize,
        budget: &mut FormatBudget,
    ) -> Result<PrettyValue, DebugError> {
        if budget.is_exhausted() {
            return Ok(PrettyValue::Truncated);
        }

        // MVP: print a summary. Deep iteration requires stable hashbrown layout.
        Ok(PrettyValue::Raw("HashMap { ... }".to_string()))
    }
}
