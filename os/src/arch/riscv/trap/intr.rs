use num_enum::TryFromPrimitive;
use riscv::register::time;

use crate::arch::SBITable;

pub const TIMER_TICK: usize = 0xfff;

#[repr(usize)]
#[derive(Debug, TryFromPrimitive)]
pub enum InterruptTypes {
    SSoftwareIntr = 1,
    STimerIntr = 5,
    SExternalIntr = 9,
}

pub fn intr_handler(intr_type: InterruptTypes) {
    match intr_type {
        InterruptTypes::STimerIntr => {
            let time = time::read();
            SBITable::set_timer(time + TIMER_TICK)
                .unwrap_or_else(|value| panic!("Unexpected timer error:{:#x}", value));
        }
        _ => {}
    }
}

pub fn timer_tick() {}
