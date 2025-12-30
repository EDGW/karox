use riscv::register::{sie, sstatus, stvec};

use crate::{arch::trap::handler::_trap_from_kernel, kserial_println};

pub mod handler;
pub mod intr;

pub fn init_trap() {
    unsafe {
        stvec::write(
            _trap_from_kernel as *const u8 as usize,
            stvec::TrapMode::Direct,
        );
        sstatus::set_sie();
        sie::set_stimer();
        sie::set_ssoft();
        sie::set_sext();
    }
    kserial_println!("Trap Handler Initialized.");
}
