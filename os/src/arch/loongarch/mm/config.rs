
/// Fixed Kernel Heap Size
pub const KERNEL_HEAP_SIZE:usize    = 128 * 0x10_0000;  // 128MiB

/// Fixed Kernel Stack Size
pub const KERNEL_STACK_SIZE:usize   = 8 * 0x10_0000;    // 8MiB
pub const KERNEL_STACK_SHIFT: usize = 20 + 3;           // 8MiB

pub const PAGE_SIZE:usize           = 0x1000;
pub const PAGE_WIDTH:usize          = 12;

pub const PTABLE_ENTRY_COUNT:usize  = 512;

/// Max HARTs Supported
pub const MAX_HARTS:usize           = 16;

pub const KERNEL_SPACE_OFFSET:usize = 0x9000_0000_0000_0000;
pub const MMIO_OFFSET:usize         = 0x8000_0000_0000_0000;