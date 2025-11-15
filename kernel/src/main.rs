//! Karox Operating System Kernel
#![deny(warnings)]
#![deny(missing_docs)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[macro_use]
pub mod entry;


/// The main function of the kernel, called from [entry::start]
/// 
/// Before this function executes, the stack space and a boot page table should be well-prepared.
pub fn kernel_main() -> ! {
    loop {}
}

/// The panic handler
#[panic_handler]
pub fn panic_handler(_pinfo: &PanicInfo) -> !{
    loop{
        
    }
}