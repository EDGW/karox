use crate::{arch::MAX_HARTS, task::task::Task};
use alloc::sync::Arc;
use core::array;
use lazy_static::lazy_static;

pub fn idle_main() -> ! {
    loop {}
}

pub fn create_idle_task(hart_id: usize) -> Arc<Task> {
    Task::new_kernel_from_entry(idle_main as *const (), hart_id).unwrap_or_else(|err| {
        panic!(
            "Could not create idle tasks for scheduler #{:}:{:?}",
            hart_id, err
        )
    })
}

lazy_static! {
    pub static ref IDLE_TASKS: [Arc<Task>; MAX_HARTS] = array::from_fn(|idx| create_idle_task(idx));
}
