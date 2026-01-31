//! This module contains drivers and management methods for physical devices

use crate::{
    arch::{
        symbols::{_ekernel, _skernel}, //trap,
    },
    devices::device_info::DeviceInfo,
    kserial_println,
};

pub mod device_info;
pub mod mmio;
pub mod serial;

/// Load devices
pub fn load_devs(dev_info: &impl DeviceInfo) {
    kserial_println!("Loading devices...");
    if let Err(err) = dev_info.init() {
        panic!("Loading device info failed {:?}", err);
    }
    #[cfg(debug_assertions)]
    {
        print_mem_info(dev_info);
        print_cpu_info(dev_info);
    }
    kserial_println!("All Devices Loaded.");
}

/// Print Memory Info
pub fn print_mem_info(dev_info: &impl DeviceInfo) {
    let mem_info = dev_info.get_mem_info().unwrap();
    kserial_println!("General Memories:");
    let mut tot_size = 0;
    for sec in mem_info {
        kserial_println!("\t{:?} (about {:} MBytes)", sec, sec.len() / 1024 / 1024);
        tot_size += sec.len();
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

#[cfg(debug_assertions)]
/// Print Memory Info
pub fn print_cpu_info(dev_info: &impl DeviceInfo) {
    use crate::kserial_print;

    let harts = dev_info.get_hart_info().unwrap();
    kserial_print!("Hart({:}): [",harts.len());
    for hart in harts {
        kserial_print!("#{:},", hart.hart_id);
    }
    kserial_println!("]");
}

