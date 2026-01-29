use crate::{
    arch::{SBITable, trap::context::TrapContext},
    task::scheduler::schedule,
};
use riscv::register::{scause::Interrupt, time};

pub const TIMER_TICK: usize = 0xff;

pub fn intr_handler(intr_type: Interrupt, _context: &mut TrapContext) {
    match intr_type {
        Interrupt::SupervisorTimer => timer_tick(),
        _ => {}
    }
}

fn timer_tick() {
    let time = time::read();
    SBITable::set_timer(time + TIMER_TICK)
        .unwrap_or_else(|value| panic!("Unexpected timer error:{:#x}", value));
    schedule();
}
