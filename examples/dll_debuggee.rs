//! Example debuggee that dynamically loads and unloads a DLL for module tracking tests.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{FreeLibrary, LoadLibraryW};

fn main() {
    println!("Loading kernel32.dll dynamically...");

    let wide: Vec<u16> = OsStr::new("kernel32.dll")
        .encode_wide()
        .chain(Some(0))
        .collect();

    let handle: HMODULE = unsafe { LoadLibraryW(&wide) }.expect("LoadLibraryW failed");
    println!("Loaded kernel32.dll at {:?}", handle.0);

    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("Freeing kernel32.dll...");
    unsafe {
        let _ = FreeLibrary(handle);
    }

    println!("Exiting");
}
