//! This module contains drivers and management methods for physical devices

use crate::{
    arch::symbols::{_ekernel, _skernel},
    devices::device_info::DeviceInfo,
    mm,
};

pub mod device_info;
pub mod mmio;
pub mod serial;

/// Initialize devices
pub fn init(dev_info: impl DeviceInfo) {
    kserial_println!("Initializing devices...");
    if let Err(err) = dev_info.init() {
        panic!("Initializing device info failed {:?}", err);
    }
    print_mem_info(&dev_info);
    mm::init_memory(dev_info.get_mem_info().unwrap());
    kserial_println!("All Devices Initialized.");
}

/// Print Memory Info
pub fn print_mem_info(dev_info: &impl DeviceInfo) {
    let mem_info = dev_info.get_mem_info().unwrap();
    kserial_println!("General Memories:");
    let mut tot_size = 0;
    for sec in mem_info {
        kserial_println!("\t{:} (about {:} MBytes)", sec, sec.length / 1024 / 1024);
        tot_size += sec.length;
    }
    let k_start = _skernel as *const u8 as usize;
    let k_end = _ekernel as *const u8 as usize;
    kserial_println!(
        "Kernel Used (Virtual): [{:#x} - {:#x}) (about {:} MBytes)",
        k_start,
        k_end,
        (k_end - k_start) / 1024 / 1024
    );
    kserial_println!(
        "Total General Memory Size: about {:} MBytes",
        tot_size / 1024 / 1024
    );
}
