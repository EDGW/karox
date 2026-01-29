use crate::{
    arch::{MAX_HARTS, hart::get_hart_info, task::switch::__switch},
    kserial_println,
    sched::{DefaultScheduler, Scheduler},
    sync::UPSafeCell,
    task::{
        get_current_sched_context, get_current_sched_context_mut, get_current_task,
        hart::HART_INFO, task::Task,
    },
};
use alloc::sync::Arc;
use core::array;
use lazy_static::lazy_static;
use riscv::register::sscratch;

lazy_static! {
    static ref SCHEDULERS: [UPSafeCell<DefaultScheduler>; MAX_HARTS] =
        array::from_fn(|hart_id| unsafe { UPSafeCell::new(DefaultScheduler::new(hart_id)) });
}

pub fn run_tasks() -> ! {
    let hart_info = get_hart_info();

    // test 1
    add_to_current(Task::new_kernel_from_entry(test_fn_1 as *const (), hart_info.hart_id).unwrap());
    // test 2
    add_to_current(Task::new_kernel_from_entry(test_fn_2 as *const (), hart_info.hart_id).unwrap());

    loop {
        unsafe {
            let task = SCHEDULERS[hart_info.hart_id].exclusive_access().fetch_new();
            HART_INFO[hart_info.hart_id]
                .inner
                .exclusive_access()
                .running_task
                .write(task.clone());
            let cur_context = get_current_sched_context_mut();
            let next_context = task.get_task_context_ptr();
            sscratch::write(task.get_trap_context_ptr() as usize);
            __switch(cur_context, next_context);
        }
    }
}

pub fn add_to_current(task: Arc<Task>) {
    unsafe { SCHEDULERS[get_hart_info().hart_id].exclusive_access() }.add_to_ready(task);
}

pub fn schedule() {
    let tsk_info = get_current_task();
    unsafe {
        let cur = tsk_info.get_task_context_mut_ptr();
        let next = get_current_sched_context();
        __switch(cur, next);
    }
}

pub fn test_fn_1() -> ! {
    loop {
        kserial_println!("When I was Yound I listen to the radio waiting.");
    }
}

pub fn test_fn_2() -> ! {
    loop {
        kserial_println!("for my favirote song. And So I lalalalalalalala.");
    }
}
