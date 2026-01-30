use core::panic::PanicInfo;

use crate::kserial_println_unsafe;

#[panic_handler]
pub fn panic_handler(pinfo: &PanicInfo) -> ! {
    unsafe {
        kserial_println_unsafe!("[Panic] {:}", pinfo);
    }
    loop {}
}
