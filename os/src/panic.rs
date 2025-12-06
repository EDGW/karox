use core::panic::PanicInfo;

use crate::kserial_println;

#[panic_handler]
pub fn panic_handler(pinfo: &PanicInfo) -> ! {
    kserial_println!("[Panic] {:}", pinfo);
    loop {}
}
