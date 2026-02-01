//! karox Operating System
//#![deny(missing_docs)]
#![no_std]
#![no_main]
#![feature(step_trait)]

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    arch::{
        hart::{get_hart_info, wake_slave_harts},
        trap,
    },
    devices::device_info::DeviceInfo,
    task::{hart::get_all_harts, scheduler::run_tasks},
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

static COUNTER: AtomicUsize = AtomicUsize::new(0);
pub fn wait() {
    let count = get_all_harts().len();
    COUNTER.fetch_add(1, Ordering::Relaxed);
    loop {
        if COUNTER.load(Ordering::Relaxed) >= count {
            break;
        }
    }
}

/// The main function of the operating system
pub fn kernel_main(dev_info: impl DeviceInfo, slave_entry: usize) -> ! {
    kserial_println!("karox running on hart #{:}.", get_hart_info().hart_id);
    mm::heap::init_heap();
    devices::load_devs(&dev_info);
    mm::init(&dev_info);
    task::init(&dev_info);
    wake_slave_harts(slave_entry);
    trap::init();
    wait();
    run_tasks();
}

/// The main function of the operating system
pub fn kernel_slave() -> ! {
    kserial_println!("karox running on slave hart #{:}.", get_hart_info().hart_id);
    mm::init_slave();
    trap::init();
    wait();
    run_tasks();
    //loop {}
}
