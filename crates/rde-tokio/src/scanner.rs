//! Tokio runtime scanner for task inspection.

use crate::task::{AsyncTask, TaskState};
use rde_core::DebugError;
use tracing::warn;

/// Scan the debuggee for Tokio runtime tasks.
pub struct TokioScanner;

impl TokioScanner {
    pub fn new() -> Self {
        Self
    }

    /// Attempt to find and list Tokio tasks in the target process.
    ///
    /// MVP: Returns an empty list with a warning if no runtime is detected.
    pub fn list_tasks(
        &self,
        _process_id: u32,
    ) -> Result<Vec<AsyncTask>, DebugError> {
        // TODO: Implement runtime signature scanning and task enumeration.
        // For MVP, return a clear "not detected" signal.
        warn!("Tokio runtime detection not yet implemented; returning empty task list");
        Ok(vec![AsyncTask {
            task_id: 0,
            state: TaskState::Completed,
            function_name: Some("No Tokio runtime detected".to_string()),
            runtime_thread_id: None,
        }])
    }
}
