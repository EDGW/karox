use riscv::register::{
    stvec::{self},
    utvec::TrapMode,
};

use crate::arch::trap::handler::__trap_from_kernel_handler;

pub mod context;
pub mod exc;
pub mod handler;
pub mod intr;

pub fn init() {
    set_trap_handler();
    intr::init();
}

fn set_trap_handler() {
    unsafe {
        stvec::write(
            __trap_from_kernel_handler as *const () as usize,
            TrapMode::Direct,
        );
    }
}
