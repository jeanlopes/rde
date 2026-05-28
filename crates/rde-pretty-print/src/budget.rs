//! Formatting budget to prevent infinite recursion and excessive memory reads.

/// Tracks resource limits during recursive pretty printing.
#[derive(Debug, Clone, Copy)]
pub struct FormatBudget {
    /// Maximum recursion depth remaining.
    pub max_depth: u32,
    /// Maximum collection elements remaining.
    pub max_elements: u32,
    /// Maximum total bytes to read from target memory.
    pub max_bytes: usize,
}

impl Default for FormatBudget {
    fn default() -> Self {
        Self {
            max_depth: 5,
            max_elements: 100,
            max_bytes: 4096,
        }
    }
}

impl FormatBudget {
    /// Create a new budget with custom limits.
    pub fn new(max_depth: u32, max_elements: u32, max_bytes: usize) -> Self {
        Self {
            max_depth,
            max_elements,
            max_bytes,
        }
    }

    /// Decrement depth and return a child budget.
    pub fn for_child(&mut self) -> Option<Self> {
        if self.max_depth == 0 {
            None
        } else {
            Some(Self {
                max_depth: self.max_depth.saturating_sub(1),
                max_elements: self.max_elements,
                max_bytes: self.max_bytes,
            })
        }
    }

    /// Consume elements and return how many are allowed.
    pub fn take_elements(&mut self, requested: u32) -> u32 {
        let allowed = requested.min(self.max_elements);
        self.max_elements = self.max_elements.saturating_sub(allowed);
        allowed
    }

    /// Consume bytes and return how many are allowed.
    pub fn take_bytes(&mut self, requested: usize) -> usize {
        let allowed = requested.min(self.max_bytes);
        self.max_bytes = self.max_bytes.saturating_sub(allowed);
        allowed
    }

    /// Check if budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.max_depth == 0 || self.max_elements == 0 || self.max_bytes == 0
    }
}
