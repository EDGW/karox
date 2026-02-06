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

pub mod colors {
    pub const BLACK: usize = 30;
    pub const RED: usize = 31;
    pub const GREEN: usize = 32;
    pub const YELLOW: usize = 33;
    pub const BLUE: usize = 34;
    pub const MAGN: usize = 35;
    pub const LIME: usize = 36;
    pub const WHITE: usize = 37;
    pub const DEFAULT: usize = 39;

    #[macro_export]
    macro_rules! bright {
        ($color: expr) => {
            $color + 60
        };
    }

    #[macro_export]
    macro_rules! backgr {
        ($color: expr) => {
            $color + 10
        };
    }
}

pub mod styles {
    pub const RESET: usize = 0;
    pub const BOLD: usize = 1;
    pub const FAINT: usize = 2;
    pub const ITALIC: usize = 3;
    pub const UNDERL: usize = 4;
    pub const SLOW_BLINK: usize = 5;
    pub const FAST_BLINK: usize = 6;
    pub const REVERSE: usize = 7;
    pub const HIDDEN: usize = 8;
    pub const STRIKE_THRU: usize = 9;
}

#[macro_export]
macro_rules! ansi_color {
    ($style:expr, $color: expr) => {
        format_args!("\u{1B}[{};{}m", $style, $color)
    };
    ($color: expr) => {
        format_args!("\u{1B}[{}m", $color)
    };
}
