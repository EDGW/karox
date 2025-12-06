//! Arch-spec Memory Management Configurations

/// Fixed Kernel Heap Size
pub const KERNEL_HEAP_SIZE: usize = 128 * 0x10_0000; // 128MiB

/// Fixed Kernel Stack Size
pub const KERNEL_STACK_SIZE: usize = 8 * 0x10_0000; // 8MiB
/// Fixed Kernel Stack Size represented in bit shift
pub const KERNEL_STACK_SHIFT: usize = 20 + 3; // 8MiB

/// The size of a normal page
pub const PAGE_SIZE: usize = 0x1000;
/// The size of a normal page represented in bit width
pub const PAGE_WIDTH: usize = 12;

/// The number of the entries in a page table
pub const PTABLE_ENTRY_COUNT: usize = 512;

/// Max HARTs Supported
pub const MAX_HARTS: usize = 16;

/// The kernel space offset
pub const KERNEL_SPACE_OFFSET: usize = 0xffff_ffc0_0000_0000;
