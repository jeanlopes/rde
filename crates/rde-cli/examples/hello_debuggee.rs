//! Minimal target program for integration testing.

fn main() {
    println!("Hello from debuggee!");
    let mut counter = 0;
    for i in 0..5 {
        counter += i;
        println!("Iteration {i}, counter = {counter}");
    }
    println!("Debuggee exiting with counter = {counter}");
}
