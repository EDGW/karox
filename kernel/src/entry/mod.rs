//! The entry point of the kernel
//! The kernel executes from here, doing arch-specific operations, then jumping to the main function.
//! An inital boot stack for the main hart and a initial page table(maybe with large pages) should be set.
//! The code here is arch-specific
use config::{self, include_arch_files, mm::KERNEL_STACK_SIZE};

/// The kernel stack
static KERNEL_STACK:[u8; KERNEL_STACK_SIZE] = [0;KERNEL_STACK_SIZE];

include_arch_files!();