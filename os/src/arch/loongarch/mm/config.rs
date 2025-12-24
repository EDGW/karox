//! Arch-spec Memory Management Configurations

use crate::arch::mm::paging::LAPaging;

/// Fixed Kernel Heap Size
pub const KERNEL_HEAP_SIZE: usize = 128 * 0x10_0000; // 128MiB

/// Fixed Kernel Stack Size
pub const KERNEL_STACK_SIZE: usize = 128 * 0x400; // 128KiB
/// Fixed Kernel Stack Size represented in bit shift
pub const KERNEL_STACK_SHIFT: usize = 20 + 3; // 8MiB

/// The Paging strategy used
pub type Paging = LAPaging;

/// Max HARTs Supported
pub const MAX_HARTS: usize = 16;

