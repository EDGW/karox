//! ## Memory Management Module

use alloc::vec::Vec;

use crate::devices::device_info::MemoryAreaInfo;

pub mod frame;
pub mod heap;
pub mod macros;
pub mod paging;

/// Trait for defining paging modes.
pub trait PagingMode {
    /// Maximum physical address supported by the paging mode.
    const MAX_PHYSICAL_ADDR: usize;
    /// Kernel virtual address offset.
    const KERNEL_OFFSET: usize;
    /// MMIO virtual address offset.
    const MMIO_OFFSET: usize;
    /// Page size in bytes.
    const PAGE_SIZE: usize;
    /// Page width in bits.
    const PAGE_WIDTH: usize;

    /// Initializes the paging mode.
    fn init();
}

/// Initializes the memory system.
/// 
pub fn init_memory(general_mem: &Vec<MemoryAreaInfo>) {
    frame::init_frames(general_mem);
    paging::init_paging();
}
