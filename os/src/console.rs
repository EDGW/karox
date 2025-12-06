// TODO: Temporarily Used
#![allow(missing_docs)]

use core::fmt::{Arguments, Error, Write};

use crate::arch::{SBITable, SBITrait};


struct SerialOut;

impl Write for SerialOut{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        SBITable::console_putstr(s).map_err(|_| Error)?;
        Ok(())
    }
}


pub fn serial_print(args: Arguments) {
    SerialOut.write_fmt(args).unwrap();
}

#[macro_export]
/// print string macro
macro_rules! kserial_print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::serial_print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! kserial_println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::serial_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));   // Use LF instead of CR-LF
    }
}