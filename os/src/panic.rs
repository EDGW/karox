use crate::kserial_println;
use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(pinfo: &PanicInfo) -> ! {
    kserial_println!("[Panic] {:}", pinfo);
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
