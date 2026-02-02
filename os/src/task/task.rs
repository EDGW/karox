use crate::{
    arch::{KERNEL_OFFSET, MAX_HARTS, task::context::TaskContext, trap::context::TrapContext},
    mm::{frame::FrameAllocatorError, space::MemSpace, stack::KernelStack},
    sync::LocalCell,
    task::{
        hart::{HART_INFO, HartInfo},
        tid::{TaskId, alloc_tid},
    },
};
use alloc::sync::Arc;
use spin::RwLock;

#[derive(Debug)]
#[repr(C)]
pub struct Task {
    /// Kernel Stack Top, always the first member.
    /// Referenced in trap handler.
    pub kstack_top: usize,

    // Basic Info
    pub tid: TaskId,
    pub status: RwLock<TaskStatus>,

    // Memory Management
    /// Memspace of current task. For kernel tasks, the value is [None].
    pub memsp: Option<Arc<MemSpace>>,
    pub kstack: KernelStack,

    /// Inner Mutable
    pub inner: LocalCell<TaskInner>,
}

#[derive(Debug)]
#[repr(C)]
pub struct TaskInner {
    pub task_context: TaskContext,
    pub trap_context: TrapContext,
    pub hart_id: usize,
}

impl Task {
    pub fn new_kernel_from_entry(
        entry: *const (),
        hart_id: usize,
    ) -> Result<Arc<Task>, FrameAllocatorError> {
        debug_assert!(hart_id < MAX_HARTS);
        debug_assert!(entry as usize >= KERNEL_OFFSET);
        let tid = alloc_tid();
        let kstack = KernelStack::new()?;
        let kstack_top = kstack.get_stack_top();
        let inner = TaskInner {
            task_context: TaskContext::uninitialized(),
            trap_context: TrapContext::zero_from_entry(
                entry,
                hart_id,
                true,
                kstack_top,
                kstack_top,
                &HART_INFO[hart_id] as *const HartInfo as usize,
            ),
            hart_id,
        };
        let res = Arc::new(Task {
            tid: tid,
            status: RwLock::new(TaskStatus::Ready),
            memsp: None,
            kstack_top: kstack.get_stack_top(),
            kstack,
            inner: unsafe { LocalCell::new(inner) },
        });
        let trap_ctx = unsafe { res.get_trap_context_mut_ptr() };
        let mut inner_exc = unsafe { res.inner.exclusive_access() };
        inner_exc.task_context = TaskContext::return_to_task(trap_ctx, kstack_top);
        drop(inner_exc);
        Ok(res)
    }
}

/// Unsafe Methods
impl Task {
    /// Get the mutable task context ptr of this task.
    /// **This function is UP-Safe and cannot be preempted.
    ///   Wrap the function in [super::preempt::disable_preempt()] and [super::preempt::restore_preempt()] if needed**
    pub unsafe fn get_task_context_ptr(&self) -> *const TaskContext {
        unsafe { &self.inner.access().task_context }
    }

    /// Get the task context ptr of this task.
    /// **This function is UP-Safe and cannot be preempted.
    ///   Wrap the function in [super::preempt::disable_preempt()] and [super::preempt::restore_preempt()] if needed**
    pub unsafe fn get_task_context_mut_ptr(&self) -> *mut TaskContext {
        unsafe { &mut self.inner.exclusive_access().task_context }
    }

    /// Get the mutable trap context ptr of this task.
    /// **This function is UP-Safe and cannot be preempted.
    ///   Wrap the function in [super::preempt::disable_preempt()] and [super::preempt::restore_preempt()] if needed**
    pub unsafe fn get_trap_context_ptr(&self) -> *const TrapContext {
        unsafe { &self.inner.access().trap_context }
    }

    /// Get the trap context ptr of this task.
    /// **This function is UP-Safe and cannot be preempted.
    ///   Wrap the function in [super::preempt::disable_preempt()] and [super::preempt::restore_preempt()] if needed**
    pub unsafe fn get_trap_context_mut_ptr(&self) -> *mut TrapContext {
        unsafe { &mut self.inner.exclusive_access().trap_context }
    }

    /// Get the hart info of this task.
    /// **This function is UP-Safe and cannot be preempted.
    ///   Wrap the function in [super::preempt::disable_preempt()] and [super::preempt::restore_preempt()] if needed**
    pub unsafe fn get_hart_info(&self) -> &'static HartInfo {
        unsafe { &HART_INFO[self.inner.exclusive_access().hart_id] }
    }
}

impl Task {
    pub fn get_tid(&self) -> usize {
        self.tid.value()
    }
}

#[derive(Debug)]
pub enum TaskStatus {
    Running,
    Ready,
    Blocked,
}

pub fn init() {}
