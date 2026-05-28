//! Minimal debuggee for module tracking tests.
//! Any Rust binary on Windows automatically loads system DLLs,
//! so this simple program is sufficient to test module tracking.

fn main() {
    println!("DLL debuggee started");
    std::thread::sleep(std::time::Duration::from_millis(200));
    println!("DLL debuggee exiting");
}
