//! karox Operating System
//#![deny(missing_docs)]
#![no_std]
#![no_main]
#![feature(step_trait)]
#![allow(long_running_const_eval)]

use crate::{
    arch::{hart::get_current_hart_id, trap},
    dev::get_working_harts,
    task::scheduler::run_tasks,
};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

extern crate alloc;

#[macro_use]
pub mod arch;
pub mod dev;
pub mod entry;
pub mod mm;
pub mod mutex;
mod panic;
pub mod sched;
pub mod task;
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
    let count = get_working_harts().len();
    COUNTER.fetch_add(1, Ordering::Relaxed);
    loop {
        if COUNTER.load(Ordering::Relaxed) >= count {
            break;
        }
    }
}

pub fn early_init_main() {
    mm::heap::init_heap();
    logging::init();
}

/// The main function of the operating system.
/// **The entry must call [early_init_main] and wake all slave harts before entering [kernel_main];**
pub fn kernel_main() -> ! {
    debug_ex!("karox running on hart #{:}.", get_current_hart_id());
    mm::init();
    trap::init();
    dev::init();
    debug_ex!("Main hart initialized (#{:}).", get_current_hart_id());

    mark_init();
    wait_for_slave();

    run_tasks();
}

/// The main function of the operating system
pub fn kernel_slave() -> ! {
    wait_for_main();

    debug_ex!("karox running on slave hart #{:}.", get_current_hart_id());

    mm::init_slave();
    trap::init();
    debug_ex!("Slave hart initialized (#{:}).", get_current_hart_id());

    wait_for_slave();

    run_tasks();
    //loop {}
}
