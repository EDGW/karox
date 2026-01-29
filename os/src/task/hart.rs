use core::mem::MaybeUninit;

use alloc::sync::Arc;

use crate::{
    arch::{MAX_HARTS, task::context::TaskContext},
    sched::idle::IDLE_TASKS,
    sync::UPSafeCell,
    task::task::Task,
};

#[derive(Debug)]
#[repr(C)]
pub struct HartInfo {
    pub hart_id: usize,
    pub inner: UPSafeCell<HartInfoInner>,
}

#[derive(Debug)]
#[repr(C)]
pub struct HartInfoInner {
    pub running_task: MaybeUninit<Arc<Task>>,
    pub sched_context: TaskContext,
}

pub static HART_INFO: [HartInfo; MAX_HARTS] = {
    const NONE: HartInfo = HartInfo {
        hart_id: 0,
        inner: unsafe {
            UPSafeCell::new(HartInfoInner {
                running_task: MaybeUninit::uninit(),
                sched_context: TaskContext::uninitialized(),
            })
        },
    };
    let mut res = [NONE; MAX_HARTS];
    let mut i = 0;
    while i < MAX_HARTS {
        res[i].hart_id = i;
        i += 1
    }
    res
};

pub fn init() {
    for i in 0..MAX_HARTS {
        let mut inner = unsafe { HART_INFO[i].inner.exclusive_access() };
        inner.running_task.write(IDLE_TASKS[i].clone());
        drop(inner);
    }
}
