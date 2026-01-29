use crate::arch::task::context::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));
unsafe extern "C" {
    pub unsafe fn __switch(cur: *mut TaskContext, next: *const TaskContext);
}
