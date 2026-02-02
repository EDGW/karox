use core::arch::asm;

use crate::{
    arch::SbiTable,
    panic_init,
    task::hart::{HART_INFO, HartInfo},
};

pub fn init_hart_info(hart_id: usize) {
    let hart_info = &HART_INFO[hart_id] as *const HartInfo;
    unsafe {
        asm!("mv tp, {}",in(reg) hart_info);
    }
}

/// Get the hart info.
pub fn get_hart_info() -> &'static HartInfo {
    let tp_value: usize;
    unsafe {
        asm!("mv {}, tp",out(reg) tp_value);
        &*(tp_value as *const HartInfo)
    }
}

/// Call from the main hart and wake slave harts.
/// **It's available only after the task module is initialized.**
pub fn wake_slave_harts(hart_id: usize, entry: usize) {
    SbiTable::hart_start(hart_id, entry, 0)
        .unwrap_or_else(|err| panic_init!("Unable to start slave hart {:}: {:?}", hart_id, err));
}
