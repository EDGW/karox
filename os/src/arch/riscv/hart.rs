use core::arch::asm;

use crate::{arch::SbiTable, panic_init};

pub fn store_hart_id(hart_id: usize) {
    unsafe {
        asm!("mv tp, {}",in(reg) hart_id);
    }
}

/// Get the hart info.
pub fn get_current_hart_id() -> usize {
    let tp_value: usize;
    unsafe {
        asm!("mv {}, tp",out(reg) tp_value);
    }
    tp_value
}

/// Call from the main hart and wake slave harts.
/// **It's available only after the task module is initialized.**
pub fn wake_slave_harts(hart_id: usize, entry: usize) {
    SbiTable::hart_start(hart_id, entry, 0)
        .unwrap_or_else(|err| panic_init!("Unable to start slave hart {:}: {:?}", hart_id, err));
}
