use crate::{
    arch::{MAX_HARTS, task::context::TaskContext},
    sched::idle::IDLE_TASKS,
    sync::UPSafeCell,
    task::{preempt::PreemptCounter, task::Task},
};
use alloc::sync::Arc;

#[derive(Debug)]
#[repr(C)]
pub struct HartInfo {
    pub hart_id: usize,
    pub preempt: PreemptCounter,
    pub inner: UPSafeCell<HartInfoInner>,
}

/// Members wrapped in [HartInfoInner] are probably changed as [super::scheduler::schedule()] executes.
/// In task execution environments, they are guaranteed to remain unchanged.
#[derive(Debug)]
#[repr(C)]
pub struct HartInfoInner {
    pub running_task: Option<Arc<Task>>,
    pub sched_context: TaskContext,
}

pub static HART_INFO: [HartInfo; MAX_HARTS] = {
    const NONE: HartInfo = HartInfo {
        hart_id: 0,
        preempt: PreemptCounter::new(),
        inner: unsafe {
            UPSafeCell::new(HartInfoInner {
                running_task: None,
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

/// Initialize task-related members in [HART_INFO].
pub fn init() {
    for i in 0..MAX_HARTS {
        let mut inner = unsafe { HART_INFO[i].inner.exclusive_access() };
        inner.running_task = Some(IDLE_TASKS[i].clone());
        drop(inner);
    }
}
