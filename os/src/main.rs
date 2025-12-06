//! Karox Operating System
#![deny(missing_docs)]
#![no_std]
#![no_main]

use crate::{
    arch::{SBITable, SBITrait},
    devices::device_info::DeviceInfo,
};
extern crate alloc;

pub mod arch;
pub mod entry;
pub mod error;
pub mod mm;
mod panic;
#[macro_use]
pub mod console;
pub mod devices;
pub mod drivers;

/// The main function of the operating system
pub fn rust_main(_hart_id: usize, dev_tree: impl DeviceInfo) -> ! {
    kserial_println!("Initializing karox...");
    mm::heap::init_heap();
    SBITable::init();
    init_devices(dev_tree);
    loop {}
}

/// Resolve the device tree and initialize basic devices
fn init_devices(dev_tree: impl DeviceInfo) {
    kserial_println!("Initializing device tree...");
    if let Err(err) = dev_tree.init() {
        panic!("Initialize device tree failed {:?}", err);
    }
    kserial_println!("Memory info {:?}", dev_tree.get_mem_info().unwrap());
}
