#![no_std]
#![no_main]

use core::panic::PanicInfo;

use karox_api::syscall;

#[unsafe(no_mangle)]
pub fn main() {
    unsafe{
        syscall::syscall(123, [234, 555, 666]);
    }
}

#[panic_handler]
pub fn panic_handler(_pinfo: &PanicInfo) -> ! {
    loop {}
}
