use crate::{arch::hart::get_hart_info, kserial_println};
use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(pinfo: &PanicInfo) -> ! {
    kserial_println!("{:}", pinfo);
    kserial_println!("Panic on hart #{:}.", get_hart_info().hart_id);
    loop {}
}

#[macro_export]
/// Trigger panic during initialization
macro_rules! panic_init {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        panic!(
            concat!("An unexpected error occurred during kernel initialization:\n\t",$fmt)
             $(, $($arg)+)?)
    }
}
