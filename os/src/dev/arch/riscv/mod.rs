use crate::dev::{arch::{cpu::CpuDriver, sifive_plic::PLICDriver}, driver::register_driver};
use alloc::boxed::Box;

pub mod cpu;
pub mod sifive_plic;

pub fn register_drivers() {
    register_driver(Box::new(PLICDriver::new()));
    register_driver(Box::new(CpuDriver::new()));
}
