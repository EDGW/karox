//! Frame Management
use alloc::{slice, vec::Vec};
use spin::Mutex;

use crate::{
    arch::mm::{config::Paging, paging::PageNum},
    devices::device_info::MemoryAreaInfo,
    kserial_println,
    mm::{PagingMode, frame::buddy::BuddyFrameAllocator},
};

pub mod buddy;
// region: FrameAllocator traits

/// Trait for Frame Allocation
pub trait FrameAllocator: Send {
    /// Initializes the frame allocator.
    fn init(&mut self);

    /// Adds a memory area to the allocator.
    fn add_frame(&mut self, general_mem: MemoryAreaInfo);

    /// Attempts to allocate `count` frames.
    unsafe fn try_alloc(&mut self, count: usize) -> Option<PageNum>;

    /// Deallocates `count` frames starting at `ppn`.
    unsafe fn decalloc(&mut self, ppn: PageNum, count: usize);

    /// Allocates `count` frames and returns a managed frame object.
    fn alloc_managed(&mut self, count: usize) -> Result<ManagedFrames, FrameAllocatorError> {
        unsafe {
            match self.try_alloc(count) {
                Some(ppn) => Ok(ManagedFrames::new(ppn, count)),
                None => Err(FrameAllocatorError::OutOfMemory),
            }
        }
    }
}

/// Errors related to frame allocation.
#[derive(Debug)]
pub enum FrameAllocatorError {
    /// Indicates that the allocator is out of memory.
    OutOfMemory,
}
// endregion

// region: ManagedFrames
/// Represents a set of managed frames.
/// 
/// Automatically deallocates frames when dropped.
pub struct ManagedFrames {
    ppn: PageNum,
    count: usize,
}

impl Drop for ManagedFrames {
    /// Deallocates the managed frames when dropped.
    fn drop(&mut self) {
        unsafe {
            FRAME_ALLOC.lock().decalloc(self.ppn, self.count);
        }
    }
}

impl ManagedFrames {
    /// Creates a new managed frame object.
    pub unsafe fn new(ppn: PageNum, count: usize) -> ManagedFrames {
        ManagedFrames { ppn, count }
    }

    /// Returns a mutable pointer to the kernel memory if the frame
    pub unsafe fn get_kernel_ptr(&self) -> *mut u8 {
        self.ppn.physical_to_kernel().get_base_addr() as *mut u8
    }

    /// Get the frame memory as a mutable reference of the specific type
    pub unsafe fn get_ref<'a, T>(&'a self) -> &'a mut T {
        unsafe {
            let ptr = self.get_kernel_ptr();
            &mut *(ptr as *mut T)
        }
    }

    /// Get the frame memory as a mutable pointer of the specific type
    pub unsafe fn get_ptr<'a, T>(&'a self) -> *mut T {
        unsafe {
            let ptr = self.get_kernel_ptr();
            ptr as *mut T
        }
    }

    /// Get the frame memory as a mutable slice of the specific type
    pub unsafe fn get_as_arr<'a, T>(&'a self, count: usize) -> &'a mut [T] {
        unsafe {
            let ptr = self.get_kernel_ptr() as *mut T;
            slice::from_raw_parts_mut(ptr, count)
        }
    }

    /// Returns the physical page number of the frames.
    pub fn get_ppn(&self) -> PageNum {
        self.ppn
    }

    /// Returns the number of frames managed.
    pub fn get_count(&self) -> usize {
        self.count
    }
}
// endregion

// region: Allocator

/// Default Frame Allocator
pub type DefaultFrameAllocator = BuddyFrameAllocator;

/// Global Frame Allocator
pub static FRAME_ALLOC: Mutex<DefaultFrameAllocator> = Mutex::new(DefaultFrameAllocator::new());

/// Initializes the frame allocator with the given memory areas.
/// 
/// Filters out memory areas that exceed the maximum physical address.
pub fn init_frames(general_mem: &Vec<MemoryAreaInfo>) {
    for area in general_mem {
        let start = area.start;
        let end = area.start + area.length;
        let mut guard = FRAME_ALLOC.lock();
        let max_addr = Paging::MAX_PHYSICAL_ADDR;
        if end <= max_addr {
            guard.add_frame(*area);
        } else if start <= max_addr && end > max_addr {
            guard.add_frame(MemoryAreaInfo {
                start,
                length: max_addr - start,
            });
            kserial_println!(
                "Ignored unreachable memory area {:?}",
                MemoryAreaInfo {
                    start: max_addr,
                    length: end - max_addr
                }
            );
        } else {
            // start > MAX_PHYSICAL_ADDR
            kserial_println!("Ignored unreachable memory area {:?}", *area);
        }
    }
}

// endregion
