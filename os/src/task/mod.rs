use alloc::sync::Arc;

use crate::{
    arch::{hart::get_hart_info, task::context::TaskContext},
    task::task::Task,
};

pub mod hart;
pub mod scheduler;
pub mod task;
pub mod tid;

pub fn get_current_task() -> Arc<Task> {
    let inner = unsafe { get_hart_info().inner.exclusive_access() };
    let res = unsafe { inner.running_task.assume_init_ref().clone() };
    drop(inner);
    res
}

pub fn get_current_sched_context() -> *const TaskContext {
    let inner = unsafe { get_hart_info().inner.exclusive_access() };
    let res = &inner.sched_context as *const TaskContext;
    drop(inner);
    res
}

pub fn get_current_sched_context_mut() -> *mut TaskContext {
    let mut inner = unsafe { get_hart_info().inner.exclusive_access() };
    let res = &mut inner.sched_context as *mut TaskContext;
    drop(inner);
    res
}

pub fn init(){
    hart::init();
}