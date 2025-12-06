// TODO:Temporarily Used
#![allow(missing_docs)]

use alloc::boxed::Box;
use spin::Mutex;

use crate::{
    arch::SBITrait,
    drivers::serial::{Uart, ns16550a::Ns16550a},
};

static UART: Mutex<Option<Box<dyn Uart>>> = Mutex::<Option<Box<dyn Uart>>>::new(None);

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
        *guard = Some({ Box::new(Ns16550a::create(0x80000000_1fe001e0)) });
    }
}
