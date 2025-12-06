//! Memory management for risc-v architecture


use bitflags::bitflags;

use crate::arch::mm::config::{KERNEL_STACK_SIZE, MAX_HARTS};
pub mod config;
pub mod paging;

bitflags! {
    /// Memory Access Type (MAT) attributes defining ordering and caching behavior.
    pub struct MemAccessType: u8 {
        /// Strongly-Ordered Non-Cacheable memory access.
        const STRONG_NONCACHE = 0;

        /// Cacheable Coherent memory access.
        const CACHE = 1;

        /// Weakly-Ordered Non-Cacheable memory access.
        const WEAK_NONCACHE = 2;

        /// Reserved
        const RESERVED = 3;
    }
}

#[unsafe(link_section = ".bss.stack")]
pub static KERNEL_STACK:[[u8;KERNEL_STACK_SIZE];MAX_HARTS]   = [[0;KERNEL_STACK_SIZE];MAX_HARTS];

pub const fn stack_area(hart_id: usize) -> &'static [u8; KERNEL_STACK_SIZE]{
    &KERNEL_STACK[hart_id]
}

pub const fn stack_top(hart_id: usize) -> *const u8{
    unsafe{
        ((&KERNEL_STACK[hart_id]) as *const [u8] as *const u8).add(KERNEL_STACK_SIZE)
    }
}