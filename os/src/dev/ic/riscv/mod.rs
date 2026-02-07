use alloc::boxed::Box;

use crate::dev::{driver::register_driver, ic::plic::PLICDriver};

pub mod plic;
pub fn register_drivers_arch() {
    register_driver(Box::new(PLICDriver::new()));
}
