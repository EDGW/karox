use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    arch::hart::get_hart_info,
    kserial_println,
    sched::Scheduler,
    task::{get_current_task, scheduler::SCHEDULERS, task::Task},
};

pub fn add_test_tasks() {
    unsafe {
        for _ in 0..5000 {
            add_to_current(test_fn as *const ());
        }
    }
}

pub unsafe fn add_to_current(entry: *const ()) {
    let hart_id = get_hart_info().hart_id;
    unsafe { SCHEDULERS[hart_id].exclusive_access() }
        .add_to_ready(Task::new_kernel_from_entry(entry, hart_id).unwrap());
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn test_fn() -> ! {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    loop {
        kserial_println!(
            "[Test Function {:} from task(tid #{:}, hart #{:})]",
            id,
            get_current_task().get_tid(),
            get_hart_info().hart_id
        );
    }
}
