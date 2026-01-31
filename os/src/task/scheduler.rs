use crate::{
    arch::{MAX_HARTS, hart::get_hart_info, task::switch::__switch},
    sched::{DefaultScheduler, Scheduler},
    sync::UPSafeCell,
    task::{
        get_current_sched_context, get_current_sched_context_mut, get_current_task,
        hart::HART_INFO, scheduler::test::add_test_tasks,
    },
};
use core::array;
use lazy_static::lazy_static;
use riscv::register::sscratch;

#[path = "test.rs"]
pub mod test;

lazy_static! {
    static ref SCHEDULERS: [UPSafeCell<DefaultScheduler>; MAX_HARTS] =
        array::from_fn(|hart_id| unsafe { UPSafeCell::new(DefaultScheduler::new(hart_id)) });
}

pub fn run_tasks() -> ! {
    let hart_info = get_hart_info();
    add_test_tasks();
    loop {
        unsafe {
            let task = SCHEDULERS[hart_info.hart_id].exclusive_access().fetch_new();
            HART_INFO[hart_info.hart_id]
                .inner
                .exclusive_access()
                .running_task = Some(task.clone());
            let cur_context = get_current_sched_context_mut();
            let next_context = task.get_task_context_ptr();
            sscratch::write(task.get_trap_context_ptr() as usize); // set sscratch
            __switch(cur_context, next_context);
        }
    }
}

/// Schedule. **Make sure interrupt is disabled before you call the scheduler**
pub fn schedule() {
    let hart_info = get_hart_info();
    if !hart_info.preempt.is_preempt_allowed() {
        hart_info.preempt.need_reschedule();
        return;
    }
    let tsk_info = get_current_task();
    unsafe {
        let cur = tsk_info.get_task_context_mut_ptr();
        let next = get_current_sched_context();
        __switch(cur, next);
    }
}
