use crate::{devices::device_info::DeviceInfo, panic_init};

pub mod config;
pub mod frame;
pub mod heap;
pub mod paging;
pub mod space;
pub mod stack;

/// Initializes the memory management module.
pub fn init(device_info: &impl DeviceInfo) {
    frame::init(
        device_info
            .get_mem_info()
            .unwrap_or_else(|err| panic_init!("Error getting memory info {:?}", err)),
    );
    paging::init();
}

/// Initializes the memory management module as a slave hart.
pub fn init_slave() {
    paging::init();
}
