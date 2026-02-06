use core::array;

use crate::{
    arch::{MAX_HARTS, hart::get_current_hart_id, task::context::TaskContext},
    dev::get_working_harts,
    sched::idle::IDLE_TASKS,
    task::{task::Task},
};
use alloc::sync::Arc;
use lazy_static::lazy_static;
use utils::sync::LocalCell;

#[derive(Debug)]
#[repr(C)]
pub struct Processor {
    pub inner: LocalCell<ProcessorInner>,
}
#[derive(Debug)]
pub struct ProcessorInner {
    pub running_task: Option<Arc<Task>>,
    pub sched_context: TaskContext,
}

lazy_static! {
    pub static ref PROCESSORS: [Processor; MAX_HARTS] = create_processor_info();
}

/// Initialize task-related members in [HART_INFO].
fn create_processor_info() -> [Processor; MAX_HARTS] {
    let res = array::from_fn(|_| Processor {
        inner: unsafe {
            LocalCell::new(ProcessorInner {
                running_task: None,
                sched_context: TaskContext::uninitialized(),
            })
        },
    });
    let harts = get_working_harts();
    for hart in harts {
        let hart_id = hart.hart_id;
        unsafe { res[hart_id].inner.exclusive_access() }.running_task =
            Some(IDLE_TASKS[hart_id].clone());
    }
    res
}

pub fn get_current_processor_context() -> &'static Processor {
    &PROCESSORS[get_current_hart_id()]
}
