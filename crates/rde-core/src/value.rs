//! Pretty-printed value representations shared across crates.

/// The result of pretty-printing a value from the debuggee.
#[derive(Debug, Clone, PartialEq)]
pub enum PrettyValue {
    /// A primitive or simple scalar value (e.g., `42`, `"hello"`).
    Scalar(String),
    /// An enum variant with optional payload (e.g., `Some(Box<PrettyValue>)`).
    Enum {
        name: String,
        payload: Option<Box<PrettyValue>>,
    },
    /// An ordered sequence of values (e.g., `[10, 20, 30]`).
    Sequence(Vec<PrettyValue>),
    /// A map of key-value pairs.
    Map(Vec<(PrettyValue, PrettyValue)>),
    /// Fallback raw memory representation when no printer matches.
    Raw(String),
    /// Indicator that the formatting budget was exhausted.
    Truncated,
}
