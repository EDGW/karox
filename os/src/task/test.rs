use core::sync::atomic::{AtomicUsize, Ordering};

use log::info;

use crate::{
    arch::hart::get_current_hart_id,
    debug_ex,
    sched::Scheduler,
    task::{get_current_task, scheduler::SCHEDULERS, task::Task},
};

pub fn add_test_tasks() {
    debug_ex!("Adding test tasks...");
    unsafe {
        for _ in 0..3000 {
            add_to_current(test_fn as *const ());
        }
    }
    debug_ex!("Test tasks added...");
}

pub unsafe fn add_to_current(entry: *const ()) {
    let hart_id = get_current_hart_id();
    let mut scheduler = unsafe { SCHEDULERS[hart_id].exclusive_access() };
    scheduler.add_to_ready(Task::new_kernel_from_entry(entry, hart_id).unwrap());
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn test_fn() -> ! {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    loop {
        info!(
            "[Test Function {:>5} from task(tid #{:>5}, hart #{:})]",
            id,
            get_current_task().get_tid(),
            get_current_hart_id()
        );
    }
}
