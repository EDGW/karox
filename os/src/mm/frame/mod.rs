//! # Frame(Physical Pages) Management.
//!
//! [FrameAllocator] is the trait for frame allocators, but it's not safe.
//! It's recommended to use [LockedFrameAllocator], a thread-safe and memory-safe wrapper around any [FrameAllocator] implementation.

use crate::{
    arch::{MAX_PHYS_ADDR, mm::PageNum},
    devices::device_info::MemoryAreaInfo,
    kserial_println,
};
use alloc::{vec, vec::Vec};
use core::ops::Range;
use spin::{Mutex, MutexGuard};

mod buddy;
mod managed;
mod stack;
pub use managed::*;
// region: FrameAllocator traits

/// Trait for frame allocators that is **not promised to be safe.**
pub trait FrameAllocator: Send {
    /// Add a memory area as available frames.
    fn add_frame(&mut self, mem_area: MemoryAreaInfo);

    /// Allocate **contiguous** frames.
    /// The function is **unsafe** because unfreed frames may lead to memory leaks.
    ///
    /// Use safe allocation functions like
    /// [FrameAllocator::alloc_managed], [FrameAllocator::alloc_multiple_managed],
    /// or [FrameAllocator::alloc_range_managed] instead.
    unsafe fn alloc(&mut self, count: usize) -> Option<PageNum>;

    /// Deallocate contiguous frames.
    /// **The number must be exactly the same as allocated before, otherwise undefined behavior may occur.**
    ///
    /// Use managed objects like [Frame], [Frames], or [FrameRange] to avoid directly deallocating.
    unsafe fn dealloc(&mut self, ppn: PageNum, count: usize);
}

/// Thread-safe wrapper around a [FrameAllocator].
/// `alloc_managed`, `alloc_multiple_managed`, and `alloc_range_managed` are provided for safe allocation.
///
/// For unsafe operations, use [LockedFrameAllocator::lock] to get a mutex guard to the internal allocator.
pub struct LockedFrameAllocator<TAlloc: FrameAllocator> {
    alloc: Mutex<TAlloc>,
}
impl<TAlloc: FrameAllocator> LockedFrameAllocator<TAlloc> {
    /// Create a new locked allocator.
    #[inline(always)]
    pub const fn new(alloc: TAlloc) -> LockedFrameAllocator<TAlloc> {
        LockedFrameAllocator {
            alloc: Mutex::new(alloc),
        }
    }
    /// Manually acquire the internal lock and get a guard to the allocator.
    #[inline(always)]
    pub fn lock(&self) -> MutexGuard<'_, TAlloc> {
        self.alloc.lock()
    }

    /// Allocate a single frame safely.
    #[inline(always)]
    pub fn alloc_managed(&self) -> Result<Frame, FrameAllocatorError> {
        match unsafe { self.lock().alloc(1) } {
            Some(ppn) => unsafe { Ok(Frame::new(ppn)) },
            None => Err(FrameAllocatorError::OutOfMemory),
        }
    }
    /// Allocate multiple frames safely, **not guaranteed to be contiguous**.
    #[inline(always)]
    pub fn alloc_multiple_managed(&self, count: usize) -> Result<FrameSet, FrameAllocatorError> {
        // using binary trials has no benifits here, so we only try once.
        let mut guard = self.lock();
        // try
        if let Some(ppn) = unsafe { guard.alloc(count) } {
            return Ok(unsafe {
                FrameSet::from_pn(Range {
                    start: ppn,
                    end: ppn + count,
                })
            });
        }
        // alloc
        let mut res = vec![];
        for _ in 0..count {
            match unsafe { guard.alloc(1) } {
                Some(ppn) => res.push(ppn),
                None => {
                    return Err(FrameAllocatorError::OutOfMemory);
                }
            }
        }
        Ok(unsafe { FrameSet::from_pn(res) })
    }
    /// Allocate contiguous frames safely.
    pub fn alloc_range_managed(&self, count: usize) -> Result<FrameRange, FrameAllocatorError> {
        match unsafe { self.lock().alloc(count) } {
            Some(first) => unsafe { Ok(FrameRange::new(first, count)) },
            None => Err(FrameAllocatorError::OutOfMemory),
        }
    }
}

/// Frame allocator errors.
#[derive(Debug)]
pub enum FrameAllocatorError {
    OutOfMemory,
}
// endregion

// region: Allocator

/// Default [FrameAllocator].
pub type DefaultFrameAllocator = buddy::BuddyFrameAllocator;

/// Global allocator instance.
pub static FRAME_ALLOC: LockedFrameAllocator<DefaultFrameAllocator> =
    LockedFrameAllocator::new(DefaultFrameAllocator::new());

/// Initialize global allocator from memory areas (trim areas exceeding [Paging::MAX_PHYSICAL_ADDR]).
pub fn init(general_mem: &Vec<MemoryAreaInfo>) {
    for area in general_mem {
        let mut guard = FRAME_ALLOC.lock();
        let start = area.start;
        let end = area.end;
        let max_addr = MAX_PHYS_ADDR;
        if end <= max_addr {
            guard.add_frame(area.clone());
        } else if start <= max_addr && end > max_addr {
            guard.add_frame(MemoryAreaInfo {
                start,
                end: max_addr,
            });
            kserial_println!(
                "Ignored unreachable memory area {:?}",
                MemoryAreaInfo {
                    start: max_addr,
                    end: end
                }
            );
        } else {
            // start > MAX_PHYSICAL_ADDR
            kserial_println!("Ignored unreachable memory area {:?}", *area);
        }
    }
}

// endregion
