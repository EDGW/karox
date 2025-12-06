use buddy_system_allocator::LockedHeap;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{arch::mm::config::KERNEL_HEAP_SIZE, define_struct_aligned, kserial_println};

define_struct_aligned!(HeapSpace, [u8; KERNEL_HEAP_SIZE], 4096);

#[unsafe(link_section = ".bss")]
pub static KERNEL_HEAP: HeapSpace = HeapSpace([0; KERNEL_HEAP_SIZE]);
#[global_allocator]
pub static KERNEL_ALLOC: LockedHeap<32> = LockedHeap::empty();
pub static HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init_heap() {
    match HEAP_INITIALIZED.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
        Ok(_) => {
            let st = KERNEL_HEAP.as_ptr() as usize;
            unsafe {
                KERNEL_ALLOC.lock().init(st, KERNEL_HEAP_SIZE);
            }
            kserial_println!("Kernel Heap Space: {:#x}+{:#x}.", st, KERNEL_HEAP_SIZE);
        }
        Err(_) => {
            panic!("The kernel heap cannot be initialized twice.");
        }
    }
}
