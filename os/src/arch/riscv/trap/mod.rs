use riscv::register::{
    sie, sstatus, stvec::{self}, utvec::TrapMode
};

use crate::arch::trap::handler::__trap_from_kernel_handler;

pub mod context;
pub mod exc;
pub mod handler;
pub mod intr;

pub fn init() {
    set_trap_handler();
    set_sie_masks();
}

fn set_trap_handler() {
    unsafe {
        stvec::write(
            __trap_from_kernel_handler as *const () as usize,
            TrapMode::Direct,
        );
    }
}

fn set_sie_masks() {
    unsafe {
        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();
    }
}

pub fn enable_trap() {
    unsafe {
        sstatus::set_sie();
    }
}

pub fn disable_trap() {
    unsafe {
        sstatus::clear_sie();
    }
}
