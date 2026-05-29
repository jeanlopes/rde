//! Async task representation for Tokio runtime inspection.

/// State of a Tokio async task.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskState {
    /// Task is actively executing on a worker thread.
    Running,
    /// Task is scheduled but not currently running.
    Idle,
    /// Task is waiting on I/O or a timer.
    Sleeping,
    /// Task has finished execution.
    Completed,
}

/// Represents a Tokio task in the debugged process.
#[derive(Debug, Clone)]
pub struct AsyncTask {
    /// Unique task identifier from the Tokio runtime.
    pub task_id: u64,
    /// Current execution state.
    pub state: TaskState,
    /// Name of the async function or spawn point (when available).
    pub function_name: Option<String>,
    /// OS thread ID of the Tokio worker thread currently running the task.
    pub runtime_thread_id: Option<u32>,
}
