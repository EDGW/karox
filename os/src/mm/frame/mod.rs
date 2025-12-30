//! Frame management.
//!
//! Provides frame allocators, managed frame wrappers, and initialization
//! helpers for physical memory frames.

use core::slice::Iter;

use crate::{
    arch::mm::{config::Paging, paging::PageNum},
    devices::device_info::MemoryAreaInfo,
    kserial_println,
    mm::PagingMode,
};
use alloc::{slice, vec, vec::Vec};
use spin::{Mutex, MutexGuard};

mod buddy;
mod stack;
// region: FrameAllocator traits

/// Trait for frame allocation.
///
/// Implementors provide methods to add available memory areas and to
/// allocate/deallocate physical frames represented by [PageNum].
pub trait FrameAllocator: Send {
    /// Add a memory area to the allocator.
    ///
    /// The supplied [MemoryAreaInfo] will be used as available physical pages.
    fn add_frame(&mut self, general_mem: MemoryAreaInfo);

    /// Try to allocate `count` contiguous frames, returning the first [PageNum] on success.
    unsafe fn try_alloc(&mut self, count: usize) -> Option<PageNum>;

    /// Deallocate `count` frames starting at `ppn`.
    unsafe fn decalloc(&mut self, ppn: PageNum, count: usize);

    /// Allocate a single managed frame and return a [ManagedFrame].
    #[inline(always)]
    fn alloc_managed(&mut self) -> Result<ManagedFrame, FrameAllocatorError> {
        match unsafe { self.try_alloc(1) } {
            Some(ppn) => unsafe { Ok(ManagedFrame::new(ppn)) },
            None => Err(FrameAllocatorError::OutOfMemory),
        }
    }
    /// Allocate `count` non-contiguous frames and return them as [ManagedFrames].
    #[inline(always)]
    fn alloc_multiple_managed(
        &mut self,
        count: usize,
    ) -> Result<ManagedFrames, FrameAllocatorError> {
        let mut res = vec![];
        for _ in 0..count {
            match unsafe { self.try_alloc(1) } {
                Some(ppn) => res.push(ppn),
                None => {
                    return Err(FrameAllocatorError::OutOfMemory);
                }
            }
        }
        Ok(ManagedFrames::new(
            res.iter()
                .map(|ppn| unsafe { ManagedFrame::new(*ppn) })
                .collect(),
        ))
    }
}

/// A thread-safe wrapper around a [FrameAllocator].
pub struct LockedFrameAllocator<TAlloc: FrameAllocator> {
    alloc: Mutex<TAlloc>,
}

impl<TAlloc: FrameAllocator> LockedFrameAllocator<TAlloc> {
    /// Create a new locked allocator from a concrete [FrameAllocator].
    #[inline(always)]
    pub const fn new(alloc: TAlloc) -> LockedFrameAllocator<TAlloc> {
        LockedFrameAllocator {
            alloc: Mutex::new(alloc),
        }
    }
    /// Manually acquire the internal lock and get a guard to the allocator.
    #[inline(always)]
    pub fn manually_lock(&self) -> MutexGuard<'_, TAlloc> {
        self.alloc.lock()
    }
}

impl<TAlloc: FrameAllocator> LockedFrameAllocator<TAlloc> {
    /// Allocate a single managed frame using the inner allocator.
    pub fn alloc_managed(&self) -> Result<ManagedFrame, FrameAllocatorError> {
        self.alloc.lock().alloc_managed()
    }
    /// Allocate `count` non-contiguous frames and using the inner allocator.
    #[inline(always)]
    pub fn alloc_multiple_managed(
        &self,
        count: usize,
    ) -> Result<ManagedFrames, FrameAllocatorError> {
        self.alloc.lock().alloc_multiple_managed(count)
    }
}

/// Errors produced by frame allocators.
#[derive(Debug)]
pub enum FrameAllocatorError {
    /// No memory left to satisfy allocation requests.
    OutOfMemory,
}
// endregion

// region: ManagedFrames
/// A managed physical frame.
///
/// The frame is automatically returned to the global allocator when dropped.
pub struct ManagedFrame {
    ppn: PageNum,
}

impl<'a> Drop for ManagedFrame {
    /// Return the frame to the global allocator on drop.
    fn drop(&mut self) {
        unsafe {
            FRAME_ALLOC.manually_lock().decalloc(self.ppn, 1);
        }
    }
}

impl ManagedFrame {
    /// Create a new [ManagedFrame] from a [PageNum].
    pub unsafe fn new(ppn: PageNum) -> ManagedFrame {
        ManagedFrame { ppn }
    }

    /// Return a kernel-space pointer to the start of this frame.
    pub unsafe fn get_kernel_ptr(&self) -> *mut u8 {
        self.ppn.physical_to_kernel().get_base_addr() as *mut u8
    }

    /// Get a mutable reference to the frame memory as [T].
    pub unsafe fn get_ref<'b, T>(&'b self) -> &'b mut T {
        unsafe {
            let ptr = self.get_kernel_ptr();
            &mut *(ptr as *mut T)
        }
    }

    /// Get a mutable pointer to the frame memory as [T].
    pub unsafe fn get_ptr<T>(&self) -> *mut T {
        unsafe {
            let ptr = self.get_kernel_ptr();
            ptr as *mut T
        }
    }

    /// Get a mutable slice view of the frame memory as [T] with `count` elements.
    pub unsafe fn get_as_arr<'b, T>(&'b self, count: usize) -> &'b mut [T] {
        unsafe {
            let ptr = self.get_kernel_ptr() as *mut T;
            slice::from_raw_parts_mut(ptr, count)
        }
    }

    /// Return the [PageNum] backing this frame.
    pub fn get_ppn(&self) -> PageNum {
        self.ppn
    }
}

/// A collection of non-contiguous managed frames.
pub struct ManagedFrames {
    frames: Vec<ManagedFrame>,
}

impl ManagedFrames {
    /// Create a new [ManagedFrames] from a vector of [ManagedFrame].
    pub fn new(frames: Vec<ManagedFrame>) -> ManagedFrames {
        ManagedFrames { frames: frames }
    }

    /// Return the number of frames in the collection.
    pub fn get_count(&self) -> usize {
        self.frames.len()
    }
    /// Return a kernel-space pointer to the frame at `index`.
    pub unsafe fn get_kernel_ptr(&self, index: usize) -> *mut u8 {
        unsafe {
            let frame = &self.frames[index];
            frame.get_kernel_ptr()
        }
    }

    /// Get a mutable reference to the frame at `index` as [T].
    pub unsafe fn get_ref<'a, T>(&'a self, index: usize) -> &'a mut T {
        unsafe {
            let frame = &self.frames[index];
            frame.get_ref()
        }
    }

    /// Get a mutable pointer to the frame at `index` as [T].
    pub unsafe fn get_ptr<'a, T>(&'a self, index: usize) -> *mut T {
        unsafe {
            let frame = &self.frames[index];
            frame.get_ptr()
        }
    }

    /// Get the [PageNum] of the frame at `index`.
    pub fn get_ppn(&self, index: usize) -> PageNum {
        let frame = &self.frames[index];
        frame.get_ppn()
    }
    /// Add a [ManagedFrame] into the collection.
    pub fn add_frame(&mut self, frame: ManagedFrame) {
        self.frames.push(frame);
    }

    /// Iterate over all managed frames.
    pub fn iter(&self) -> Iter<'_, ManagedFrame> {
        self.frames.iter()
    }
}
// endregion

// region: Allocator

/// Default [FrameAllocator] type alias.
pub type DefaultFrameAllocator = buddy::BuddyFrameAllocator;

/// Global frame allocator instance.
pub static FRAME_ALLOC: LockedFrameAllocator<DefaultFrameAllocator> =
    LockedFrameAllocator::new(DefaultFrameAllocator::new());

/// Initialize the global frame allocator from a list of memory areas.
///
/// Areas that exceed [Paging]::MAX_PHYSICAL_ADDR are trimmed or ignored.
pub fn init_frames(general_mem: &Vec<MemoryAreaInfo>) {
    for area in general_mem {
        let mut guard = FRAME_ALLOC.manually_lock();
        let start = area.start;
        let end = area.start + area.length;
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
