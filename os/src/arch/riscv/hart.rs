use core::arch::asm;

use crate::{
    arch::{SbiTable, riscv::sbi::HartStatus},
    panic_init,
    task::hart::{HART_INFO, HartInfo, get_all_harts},
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
pub fn wake_slave_harts(slave_entry: usize) {
    let current = get_hart_info();
    for hart in get_all_harts() {
        if hart.hart_id != current.hart_id {
            SbiTable::hart_start(hart.hart_id, slave_entry, 0).unwrap_or_else(|err| {
                panic_init!("Unable to start slave hart {:}: {:?}", hart.hart_id, err)
            });
        }
    }

    for hart in get_all_harts() {
        loop {
            match SbiTable::hart_get_status(hart.hart_id).unwrap() {
                HartStatus::Started => break,
                HartStatus::StartPending => {}
                status => panic_init!(
                    "Unable to start slave hart {:}: Invalid status: {:?}",
                    hart.hart_id,
                    status
                ),
            }
        }
    }
}
