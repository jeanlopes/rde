//! Integration test: Tokio scanner returns placeholder on missing runtime.

use rde_tokio::scanner::TokioScanner;
use rde_tokio::task::TaskState;

#[test]
fn test_tokio_scanner_no_runtime() {
    let scanner = TokioScanner::new();
    let tasks = scanner.list_tasks(1234).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_id, 0);
    assert_eq!(tasks[0].state, TaskState::Completed);
    assert_eq!(tasks[0].function_name, Some("No Tokio runtime detected".to_string()));
}
