//! ## Memory Management Module

use crate::devices::device_info::MemoryAreaInfo;
use alloc::vec::Vec;

pub mod config;
pub mod frame;
pub mod heap;
pub mod paging;
pub mod space;
pub mod stack;

/// Initializes the memory system.
///
pub fn init(general_mem_areas: &Vec<MemoryAreaInfo>) {
    frame::init(general_mem_areas);
    paging::init();
}
