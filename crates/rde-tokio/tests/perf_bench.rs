//! Performance benchmark: Tokio task listing.

use rde_tokio::scanner::TokioScanner;
use std::time::Instant;

#[test]
fn bench_tokio_task_listing() {
    let scanner = TokioScanner::new();
    let start = Instant::now();
    let tasks = scanner.list_tasks(1234).unwrap();
    let elapsed = start.elapsed();

    println!("Tokio task listing (no runtime): {:?} — {} tasks", elapsed, tasks.len());
    assert!(elapsed.as_secs_f64() < 2.0, "Task listing took too long: {:?}", elapsed);
}
