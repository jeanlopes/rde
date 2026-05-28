//! Printer registry for looking up pretty printers by type name.

use crate::{FormatBudget, MemoryReader, PrettyValue};
use rde_core::DebugError;
use std::collections::HashMap;

/// Trait for pretty-printing a specific Rust type.
pub trait PrettyPrinter: Send + Sync {
    /// Format the value at `address` in the target process.
    fn format(
        &self,
        reader: &dyn MemoryReader,
        address: usize,
        budget: &mut FormatBudget,
    ) -> Result<PrettyValue, DebugError>;
}

/// Registry mapping type names to pretty printers.
pub struct PrinterRegistry {
    printers: HashMap<String, Box<dyn PrettyPrinter>>,
}

impl Default for PrinterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PrinterRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            printers: HashMap::new(),
        }
    }

    /// Register a pretty printer for a type name.
    pub fn register(&mut self, type_name: &str, printer: Box<dyn PrettyPrinter>) {
        self.printers.insert(type_name.to_string(), printer);
    }

    /// Look up a pretty printer by exact type name.
    pub fn lookup(&self, type_name: &str) -> Option<&dyn PrettyPrinter> {
        self.printers.get(type_name).map(|p| p.as_ref())
    }
}
