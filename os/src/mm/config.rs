use crate::{
    arch::{PAGE_WIDTH, mm::paging::PageDir},
    mm::paging::PageDirTrait,
};

pub const KERNEL_HEAP_SIZE: usize = 128 * 0x10_0000; // 128MiB

pub const KERNEL_STACK_PAGES: usize = 32; // 128KiB
pub const KERNEL_STACK_SIZE: usize = KERNEL_STACK_PAGES * PAGE_SIZE; // 128KiB
pub const KERNEL_STACK_SHIFT: usize = 17; // 128KiB

pub const PAGE_SIZE: usize = 1 << PAGE_WIDTH;

pub const PTABLE_ENTRY_COUNT: usize = 1 << PageDir::LEVEL_WIDTH;
