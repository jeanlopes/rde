//! RDE Pretty Print — Human-readable formatting for Rust standard types.

pub mod budget;
pub mod printers;
pub mod registry;

pub use budget::FormatBudget;
pub use registry::{PrinterRegistry, PrettyPrinter};

pub use rde_core::memory::MemoryReader;
pub use rde_core::value::PrettyValue;

use printers::{hashmap::HashMapPrinter, option::OptionPrinter, result::ResultPrinter, string::StringPrinter, vec::VecPrinter};

/// Create a registry with all built-in pretty printers pre-registered.
pub fn built_in_registry() -> PrinterRegistry {
    let mut reg = PrinterRegistry::new();
    reg.register("core::option::Option", Box::new(OptionPrinter));
    reg.register("std::option::Option", Box::new(OptionPrinter));
    reg.register("core::vec::Vec", Box::new(VecPrinter));
    reg.register("std::vec::Vec", Box::new(VecPrinter));
    reg.register("alloc::vec::Vec", Box::new(VecPrinter));
    reg.register("core::string::String", Box::new(StringPrinter));
    reg.register("std::string::String", Box::new(StringPrinter));
    reg.register("alloc::string::String", Box::new(StringPrinter));
    reg.register("std::collections::hash::map::HashMap", Box::new(HashMapPrinter));
    reg.register("std::collections::HashMap", Box::new(HashMapPrinter));
    reg.register("core::result::Result", Box::new(ResultPrinter));
    reg.register("std::result::Result", Box::new(ResultPrinter));
    reg
}
