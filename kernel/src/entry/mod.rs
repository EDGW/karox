//! The entry point of the kernel
//! The kernel executes from here, doing arch-specific operations, then jumping to the main function.
//! An inital boot stack for the main hart and a initial page table(maybe with large pages) should be set.
//! The code here is arch-specific


use core::fmt::{Arguments, Write};

use config::{self, include_arch_files, mm::KERNEL_STACK_SIZE};

/// The kernel stack
#[unsafe(link_section = ".bss.stack")]
static KERNEL_STACK:[u8; KERNEL_STACK_SIZE] = [0;KERNEL_STACK_SIZE];

include_arch_files!();

/// The serial output struct
pub struct SerialOut;

impl Write for SerialOut{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        kserial_output(s);
        Ok(())
    }
}

/// Print formmated string to the serial port
pub fn kwrite_fmt(fmt: Arguments){
    SerialOut.write_fmt(fmt).unwrap();
}

/// Print formmated string to the serial port
/// 
/// Its only for kernel use
#[macro_export]
macro_rules! kprintln {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::entry::kwrite_fmt(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}