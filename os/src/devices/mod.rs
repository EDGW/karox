//! This module contains drivers and management methods for physical devices

use crate::{devices::device_info::DeviceInfo, panic_init};

pub mod device_info;
pub mod mmio;
pub mod serial;

/// Load devices
pub fn load_devs(dev_info: &impl DeviceInfo) {
    if let Err(err) = dev_info.init() {
        panic_init!("Error Loading devices: {:?}", err);
    }
}
