//! karox Operating System
//#![deny(missing_docs)]
#![no_std]
#![no_main]
#![feature(step_trait)]

use crate::{
    arch::{
        SBITable, SBITrait,
        hart::{get_hart_info, init_hart_info},
        trap,
    },
    devices::device_info::DeviceInfo,
    task::scheduler::run_tasks,
};
extern crate alloc;

pub mod arch;
pub mod devices;
pub mod entry;
pub mod mm;
pub mod mutex;
mod panic;
pub mod sched;
pub mod sync;
pub mod task;
pub mod utils;
#[macro_use]
pub mod console;

/// The main function of the operating system
pub fn rust_main(hart_id: usize, dev_info: impl DeviceInfo) -> ! {
    init_hart_info(hart_id);
    mm::heap::init_heap();
    SBITable::init();
    devices::load_devs(&dev_info);
    mm::init(dev_info.get_mem_info().unwrap());
    kserial_println!("karox running on hart #{:}", get_hart_info().hart_id);
    task::init();
    trap::init();
    run_tasks();
}
