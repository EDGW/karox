use crate::{
    arch::trap::{context::TrapContext, handler::__return_to_task},
    mm::stack::KernelStack,
};
use core::fmt::Debug;

/// Task context needed to switch between execution flows.
/// **The order of the members of [TaskContext] is fixed and will be referenced in [crate::arch::task::switch::__switch]**
#[repr(C)]
pub struct TaskContext {
    /// Return Address
    ra: usize,
    /// Stack Pointer
    sp: usize,
    /// Callee-Saved GPRs s0 - s11
    s: [usize; 12],
}

impl TaskContext {
    pub const fn uninitialized() -> TaskContext {
        TaskContext {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    pub fn zero_from_entry(entry: *const (), stack: &KernelStack) -> TaskContext {
        TaskContext {
            ra: entry as usize,
            sp: stack.get_stack_top(),
            s: [0; 12],
        }
    }
    pub fn return_to_task(trap_context: *mut TrapContext, stack_top: usize) -> TaskContext {
        TaskContext {
            ra: __return_to_task as *const () as usize,
            sp: stack_top,
            s: {
                let mut s = [0; 12];
                s[1] = trap_context as usize; // s1: arg
                s
            },
        }
    }
}

impl Debug for TaskContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaskContext")
            .field("ra", &format_args!("{:#x}", self.ra))
            .field("sp", &format_args!("{:#x}", self.sp))
            .field("s", &self.s)
            .finish()
    }
}
