use alloc::boxed::Box;

use crate::dev::{driver::register_driver, serial::uart16550::Uart16550Driver};

pub mod uart16550;

pub trait Uart: Send {
    fn read(&mut self) -> Option<u8>;
    fn write(&mut self, word: u8);
    fn flush(&mut self);
}

pub fn register_drivers() {
    register_driver(Box::new(Uart16550Driver));
}
