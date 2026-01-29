use crate::arch::trap::{context::TrapContext, exc::exception_handler, intr::intr_handler};
use core::arch::global_asm;
use riscv::register::scause::Interrupt;

unsafe extern "C" {
    pub unsafe fn __trap_from_kernel_handler();
    /// The argument is given in s1 register
    pub unsafe fn __return_to_task(/*trap_context: *const TrapContext*/);
}

global_asm!(include_str!("handler.S"));

#[unsafe(no_mangle)]
pub extern "C" fn __handler(context: &mut TrapContext, scause: usize, stval: usize) {
    const CODE_MASK: usize = usize::MAX >> 1;
    let is_intr = (scause & !CODE_MASK) != 0;
    let code = scause & CODE_MASK;
    if is_intr {
        intr_handler(Interrupt::from(code), context);
    } else {
        exception_handler(code, context, stval);
    }
}
