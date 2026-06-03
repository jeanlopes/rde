use windows::Win32::System::Threading::{CreateProcessW, CREATE_SUSPENDED, STARTUPINFOW, PROCESS_INFORMATION};
use windows::Win32::System::Diagnostics::Debug::{GetThreadContext, SetThreadContext, CONTEXT, CONTEXT_ALL_AMD64};
use windows::Win32::Foundation::CloseHandle;
use windows::core::PWSTR;

fn main() {
    unsafe {
        let mut cmd: Vec<u16> = "notepad.exe\0".encode_utf16().collect();
        let mut si: STARTUPINFOW = std::mem::zeroed();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
        
        let result = CreateProcessW(
            None,
            PWSTR(cmd.as_mut_ptr()),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            None,
            &si,
            &mut pi,
        );
        
        if result.is_ok() {
            // Stack-allocated (might be unaligned)
            let mut ctx_stack = CONTEXT::default();
            ctx_stack.ContextFlags = CONTEXT_ALL_AMD64;
            let r1 = GetThreadContext(pi.hThread, &mut ctx_stack);
            println!("Stack GetThreadContext: {:?}", r1);
            if r1.is_ok() {
                println!("Stack Original RIP: 0x{:x}", ctx_stack.Rip);
            }
            
            // Aligned allocation
            let layout = std::alloc::Layout::from_size_align(std::mem::size_of::<CONTEXT>(), 16).unwrap();
            let ptr = std::alloc::alloc(layout) as *mut CONTEXT;
            std::ptr::write_bytes(ptr as *mut u8, 0, std::mem::size_of::<CONTEXT>());
            
            (*ptr).ContextFlags = CONTEXT_ALL_AMD64;
            let r2 = GetThreadContext(pi.hThread, ptr);
            println!("Aligned GetThreadContext: {:?}", r2);
            if r2.is_ok() {
                println!("Aligned Original RIP: 0x{:x}", (*ptr).Rip);
                
                let original_rip = (*ptr).Rip;
                (*ptr).Rip = 0x7FF66905BDCC;
                let r3 = SetThreadContext(pi.hThread, ptr);
                println!("Aligned SetThreadContext: {:?}", r3);
                
                std::ptr::write_bytes(ptr as *mut u8, 0, std::mem::size_of::<CONTEXT>());
                (*ptr).ContextFlags = CONTEXT_ALL_AMD64;
                let r4 = GetThreadContext(pi.hThread, ptr);
                println!("Aligned GetThreadContext2: {:?}", r4);
                if r4.is_ok() {
                    println!("Aligned After SetThreadContext: RIP=0x{:x} (expected 0x7FF66905BDCC)", (*ptr).Rip);
                }
                
                // Restore
                (*ptr).Rip = original_rip;
                let _ = SetThreadContext(pi.hThread, ptr);
            }
            
            std::alloc::dealloc(ptr as *mut u8, layout);
            let _ = CloseHandle(pi.hThread);
            let _ = CloseHandle(pi.hProcess);
        }
    }
}
