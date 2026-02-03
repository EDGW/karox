// TODO: Temporarily Used
#![allow(missing_docs)]

use crate::{arch::SbiTable, mutex::SpinLock};
use core::fmt::{Arguments, Error, Write};

static CON_LOCK: SpinLock<()> = SpinLock::new(());

struct SerialOut;

impl Write for SerialOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.as_bytes() {
            SbiTable::console_putchr(*c as char).map_err(|_| Error)?;
        }
        Ok(())
    }
}

pub fn serial_print(args: Arguments) {
    let guard = CON_LOCK.lock_no_preempt();
    SerialOut.write_fmt(args).unwrap();
    drop(guard);
}

pub unsafe fn serial_print_unsafe(args: Arguments) {
    SerialOut.write_fmt(args).unwrap();
}

#[macro_export]
/// print string macro
macro_rules! kserial_print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::serial_print(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! kserial_println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::serial_print(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?))   // Use CR-LF to adapt to QEMU
    }
}

#[macro_export]
/// print string macro
macro_rules! kserial_print_unsafe {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::serial_print_unsafe(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! kserial_println_unsafe {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::serial_print_unsafe(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?))   // Use CR-LF to adapt to QEMU
    }
}
