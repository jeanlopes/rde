//! Example debuggee that spawns worker threads for thread tracking tests.

use std::thread;
use std::time::Duration;

fn main() {
    println!("Main thread started");

    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::spawn(move || {
                println!("Worker {i} started");
                thread::sleep(Duration::from_millis(50));
                println!("Worker {i} done");
            })
        })
        .collect();

    for h in handles {
        let _ = h.join();
    }

    println!("Main thread exiting");
}
