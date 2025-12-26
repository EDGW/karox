//! This module provides the heap object allocation functions for the kernel
//!
//! It's the earliest-initialized module for the kernel can't run without dynamic allocation

use buddy_system_allocator::LockedHeap;

use crate::{arch::mm::config::KERNEL_HEAP_SIZE, define_struct, kserial_println};

// The packed type for heap space.
// It's a huge struct aligned to a page
define_struct!(aligned, HeapSpace, [u8; KERNEL_HEAP_SIZE], 4096);

/// The kernel heap space
#[unsafe(link_section = ".bss.heap")]
pub static KERNEL_HEAP: HeapSpace = HeapSpace([0; KERNEL_HEAP_SIZE]);

/// The global allocator, a buddy-system-allocator
#[global_allocator]
pub static KERNEL_ALLOC: LockedHeap<32> = LockedHeap::empty();

/// Initialize the kernel allocator
pub fn init_heap() {
    let st = KERNEL_HEAP.as_ptr() as usize;
    unsafe {
        KERNEL_ALLOC.lock().init(st, KERNEL_HEAP_SIZE);
    }
    kserial_println!("Kernel Heap Space: {:#x}+{:#x}.", st, KERNEL_HEAP_SIZE);
}
