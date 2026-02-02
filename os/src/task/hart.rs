use crate::{
    arch::{MAX_HARTS, task::context::TaskContext},
    devices::device_info::DeviceInfo,
    panic_init,
    sched::idle::IDLE_TASKS,
    sync::LocalCell,
    task::{preempt::PreemptCounter, task::Task},
};
use alloc::{sync::Arc, vec, vec::Vec};
use spin::Once;

#[derive(Debug)]
#[repr(C)]
pub struct HartInfo {
    pub hart_id: usize,
    pub preempt: PreemptCounter,
    pub inner: LocalCell<HartInfoInner>,
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
            LocalCell::new(HartInfoInner {
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

static HARTS: Once<Vec<&'static HartInfo>> = Once::new();

pub fn get_all_harts() -> &'static Vec<&'static HartInfo> {
    HARTS
        .get()
        .expect("Error getting all harts: Not initialized.")
}

/// Initialize task-related members in [HART_INFO].
pub fn init(dev_info: &impl DeviceInfo) {
    let harts = dev_info
        .get_hart_info()
        .unwrap_or_else(|err| panic_init!("Error getting hart info: {:?}", err));
    let mut all_harts = vec![];
    for hart in harts {
        let hart_id = hart.hart_id;
        let mut inner = unsafe { HART_INFO[hart_id].inner.exclusive_access() };
        inner.running_task = Some(IDLE_TASKS[hart_id].clone());
        drop(inner);
        all_harts.push(&HART_INFO[hart_id]);
    }
    HARTS.call_once(|| all_harts);
}
