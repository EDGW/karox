use core::arch::asm;

use crate::task::hart::{HART_INFO, HartInfo};

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
