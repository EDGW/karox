use crate::{
    arch::{SbiTable, trap::context::TrapContext},
    task::scheduler::schedule,
};
use riscv::register::{scause::Interrupt, sie, sstatus, time};

pub const TIMER_TICK: usize = 0x1000;

pub fn intr_handler(intr_type: Interrupt, _context: &mut TrapContext) {
    match intr_type {
        Interrupt::SupervisorTimer => timer_tick(),
        _ => {}
    }
}

fn timer_tick() {
    let time = time::read();
    SbiTable::set_timer(time + TIMER_TICK)
        .unwrap_or_else(|value| panic!("Unexpected timer error:{:?}", value));
    schedule();
}

fn set_sie_masks() {
    unsafe {
        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();
    }
}

pub fn enable_intr() {
    unsafe {
        sstatus::set_sie();
    }
}

/// Disable interrupt and return the previous interrupt status.
pub fn disable_intr() -> bool {
    let res = sstatus::read().sie();
    unsafe {
        sstatus::clear_sie();
    }
    res
}

/// Restore interrupt status.
pub fn restore_intr(previous: bool) {
    if previous {
        unsafe {
            sstatus::set_sie();
        }
    } else {
        unsafe {
            sstatus::clear_sie();
        }
    }
}

pub fn init() {
    set_sie_masks();
}
