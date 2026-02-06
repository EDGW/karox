use crate::debug_ex;

pub mod config;
pub mod frame;
pub mod heap;
pub mod paging;
pub mod space;
pub mod stack;

/// Initializes the memory management module.
pub fn init() {
    debug_ex!("Initializing memory management module...");
    frame::init();
    paging::init();
    debug_ex!("Memory management module initialized.");
}

/// Initializes the memory management module as a slave hart.
pub fn init_slave() {
    paging::init();
}
