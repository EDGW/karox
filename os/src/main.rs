//! karox Operating System
//#![deny(missing_docs)]
#![no_std]
#![no_main]

use crate::{
    arch::{SBITable, SBITrait},
    devices::device_info::DeviceInfo,
};
extern crate alloc;

pub mod arch;
pub mod devices;
pub mod entry;
pub mod error;
pub mod mm;
mod panic;
pub mod sched;
pub mod task;
pub mod utils;
#[macro_use]
pub mod console;

/// The main function of the operating system
pub fn rust_main(_hart_id: usize, dev_info: impl DeviceInfo) -> ! {
    kserial_println!("Initializing karox...");
    mm::heap::init_heap();
    SBITable::init();
    devices::init(dev_info);
    loop {}
}
