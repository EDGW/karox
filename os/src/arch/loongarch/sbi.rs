// TODO:Temporarily Used
#![allow(missing_docs)]

use alloc::boxed::Box;

use crate::{
    arch::SBITrait,
    devices::serial::{Uart, uart16550::Uart16550},
};

static UART: SpinLock<Option<Box<dyn Uart>>> = SpinLock::<Option<Box<dyn Uart>>>::new(None);

pub struct SBITable;

impl SBITrait for SBITable {
    fn console_putstr(c: &str) -> Result<(), usize> {
        let mut guard = UART.lock();
        if let Some(uart) = guard.as_mut() {
            for chr in c.as_bytes() {
                uart.write(*chr);
            }
        }
        Ok(())
    }
    fn init() {
        let mut guard = UART.lock();
        *guard = Some(Box::new(Uart16550::create(0x80000000_1fe001e0)));
    }
}
