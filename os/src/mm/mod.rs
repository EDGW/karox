//! ## Module for memory management

use alloc::vec::Vec;

use crate::devices::device_info::MemoryAreaInfo;

pub mod heap;
pub mod frame;
pub mod macros;
pub mod types;

// Initialize the memory
pub fn init_memory(general_mem: &Vec<MemoryAreaInfo>){
    frame::init_frame(general_mem);
}