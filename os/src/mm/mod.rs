use crate::devices::device_info::DeviceInfo;

pub mod config;
pub mod frame;
pub mod heap;
pub mod paging;
pub mod space;
pub mod stack;

/// Initializes the memory management module.
pub fn init(device_info: &impl DeviceInfo) {
    frame::init(device_info.get_mem_info().unwrap());
    paging::init();
}

/// Initializes the memory management module as a slave hart.
pub fn init_slave() {
    paging::init();
}
