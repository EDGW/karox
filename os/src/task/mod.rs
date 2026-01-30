//! Task Module
//! We divide the execution environment into three types:
//! * Task Execution Environment: This is when the content of the Task is being executed. 
//!     
//!     At this time, the result returned by [get_current_task()] remains unchanged, 
//!     which points to the current task's [Arc] pointer.
//!     
//!     **When a Trap is encountered, we are still in the Task Execution Environment, 
//!     but the control flow has changed**, because the executing task itself hasn't changed.
//! 
//! * Scheduling Environment: This refers to when [scheduler::schedule()] and [scheduler::run_tasks()] are executing.
//! 
//!     [scheduler::run_tasks()] is the scheduling function, responsible for 
//!     selecting the next task to execute from the scheduler and transferring control to that task.
//!     [scheduler::schedule()] attempts to trigger a scheduling event if possible. 
//!     If scheduling is possible, it will transfer control to the scheduler; 
//!     otherwise (for example, if the hart is set to non-preemptible), it will do nothing.
//! 
//! * Initializing Environment: This refers to when the operating system is still initializing.

use crate::{
    arch::{hart::get_hart_info, task::context::TaskContext},
    task::{preempt::{disable_preempt, restore_preempt}, task::Task},
};
use alloc::sync::Arc;

pub mod hart;
pub mod preempt;
pub mod scheduler;
pub mod task;
pub mod tid;

/// Get the current running task.
pub fn get_current_task() -> Arc<Task> {
    disable_preempt();
    let inner = unsafe { get_hart_info().inner.access() };
    let res = inner.running_task.as_ref().unwrap().clone();
    drop(inner);
    restore_preempt();
    res
}

/// Get a pointer to the scheduling context of the current hart.
pub fn get_current_sched_context() -> *const TaskContext {
    disable_preempt();
    let inner = unsafe { get_hart_info().inner.access() };
    let res = &inner.sched_context as *const TaskContext;
    drop(inner);
    restore_preempt();
    res
}

/// Get a mutable pointer to the scheduling context of the current hart.
pub fn get_current_sched_context_mut() -> *mut TaskContext {
    disable_preempt();
    let mut inner = unsafe { get_hart_info().inner.exclusive_access() };
    let res = &mut inner.sched_context as *mut TaskContext;
    drop(inner);
    restore_preempt();
    res
}

/// Initialize
pub fn init() {
    hart::init();
}
