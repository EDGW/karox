//! karox Operating System
//#![deny(missing_docs)]
#![no_std]
#![no_main]
#![feature(step_trait)]

use crate::{
    arch::{hart::get_hart_info, trap},
    devices::device_info::DeviceInfo,
    task::{hart::get_all_harts, scheduler::run_tasks},
};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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
#[macro_use]
pub mod logging;

static MAIN_INITIALIZED: AtomicBool = AtomicBool::new(false);

fn mark_init() {
    MAIN_INITIALIZED.store(true, Ordering::Relaxed);
}

fn wait_for_main() {
    while !MAIN_INITIALIZED.load(Ordering::Relaxed) {}
}

fn wait_for_slave() {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = get_all_harts().len();
    COUNTER.fetch_add(1, Ordering::Relaxed);
    loop {
        if COUNTER.load(Ordering::Relaxed) >= count {
            break;
        }
    }
}

pub fn early_init_main(dev_info: &impl DeviceInfo) {
    mm::heap::init_heap();
    logging::init();
    devices::load_devs(dev_info);
}

/// The main function of the operating system.
/// **The entry must call [early_init_main] and wake all slave harts before entering [kernel_main];**
pub fn kernel_main(dev_info: impl DeviceInfo) -> ! {
    debug_ex!("karox running on hart #{:}.", get_hart_info().hart_id);
    mm::init(&dev_info);
    task::init(&dev_info);
    trap::init();

    mark_init();
    wait_for_slave();

    run_tasks();
}

/// The main function of the operating system
pub fn kernel_slave() -> ! {
    wait_for_main();

    debug_ex!("karox running on slave hart #{:}.", get_hart_info().hart_id);

    mm::init_slave();
    trap::init();

    wait_for_slave();

    run_tasks();
    //loop {}
}
