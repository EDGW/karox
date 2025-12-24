//! Frame Management

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

/// Trait for Frame Allocation
pub trait FrameAllocator: Send {
    /// Adds memory area to allocator.
    fn add_frame(&mut self, general_mem: MemoryAreaInfo);

    /// Allocates frames, returning [PageNum] if successful.
    unsafe fn try_alloc(&mut self, count: usize) -> Option<PageNum>;

    /// Deallocates frames starting at [PageNum].
    unsafe fn decalloc(&mut self, ppn: PageNum, count: usize);

    /// Allocates contiguous frames
    #[inline(always)]
    fn alloc_contiguous_managed(
        &mut self,
        count: usize,
    ) -> Result<ManagedFrameRange, FrameAllocatorError> {
        unsafe {
            match self.try_alloc(count) {
                Some(ppn) => Ok(ManagedFrameRange::new(ppn, count)),
                None => Err(FrameAllocatorError::OutOfMemory),
            }
        }
    }
    /// Allocates non-contiguous frames
    #[inline(always)]
    fn alloc_managed(&mut self, count: usize) -> Result<ManagedFrames, FrameAllocatorError> {
        let mut cnt = count;
        let mut res = vec![];
        /*
         * NOTE: [ManagedFrameRange] objects must only be created after pages has been
         * successfully allocated.
         *
         * If [ManagedFrameRange] instances are created during the allocation process
         * and an out-of-memory error occurs, the function will attempt to drop these
         * partially created objects. Dropping [ManagedFrameRange] requires acquiring
         * the allocator lock, which can result in a deadlock.
         */
        while cnt != 0 {
            let mut trial = cnt;
            while trial != 0 {
                unsafe {
                    match self.try_alloc(trial) {
                        Some(ppn) => {
                            res.push((ppn, trial));
                            break;
                        }
                        None => {
                            trial /= 2;
                        }
                    }
                }
            }
            if trial == 0 {
                // resume
                for r in res {
                    unsafe {
                        self.decalloc(r.0, r.1);
                    }
                }
                return Err(FrameAllocatorError::OutOfMemory);
            }
            cnt -= trial;
        }
        Ok(ManagedFrames::new(
            res.iter()
                .map(|r| unsafe { ManagedFrameRange::new(r.0, r.1) })
                .collect(),
        ))
    }
}

/// A thread-safe locked frame allocator packer
pub struct LockedFrameAllocator<TAlloc: FrameAllocator> {
    alloc: Mutex<TAlloc>,
}

impl<TAlloc: FrameAllocator> LockedFrameAllocator<TAlloc> {
    /// Create a locked frame allocator from a specific allocator
    #[inline(always)]
    pub const fn new(alloc: TAlloc) -> LockedFrameAllocator<TAlloc> {
        LockedFrameAllocator {
            alloc: Mutex::new(alloc),
        }
    }
    #[inline(always)]
    pub fn manually_lock(&self) -> MutexGuard<'_, TAlloc> {
        self.alloc.lock()
    }
}

impl<TAlloc: FrameAllocator> LockedFrameAllocator<TAlloc> {
    pub fn alloc_managed(&self, count: usize) -> Result<ManagedFrames, FrameAllocatorError> {
        self.alloc.lock().alloc_managed(count)
    }

    pub fn alloc_contiguous_managed(
        &self,
        count: usize,
    ) -> Result<ManagedFrameRange, FrameAllocatorError> {
        self.alloc.lock().alloc_contiguous_managed(count)
    }
}

/// Frame allocation errors.
#[derive(Debug)]
pub enum FrameAllocatorError {
    /// Allocator has exhausted available memory.
    OutOfMemory,
}
// endregion

// region: ManagedFrames
/// Represents a set of managed frames that is contiguous in physical memory.
///
/// Automatically deallocates frames when dropped.
pub struct ManagedFrameRange {
    ppn: PageNum,
    count: usize,
}

impl<'a> Drop for ManagedFrameRange {
    /// Automatically deallocates frames on drop.
    fn drop(&mut self) {
        unsafe {
            FRAME_ALLOC.manually_lock().decalloc(self.ppn, self.count);
        }
    }
}

impl ManagedFrameRange {
    /// Creates new [ManagedFrames] from [PageNum] and count.
    pub unsafe fn new(ppn: PageNum, count: usize) -> ManagedFrameRange {
        ManagedFrameRange { ppn, count }
    }

    /// Returns a mutable pointer to the kernel memory if the frame
    pub unsafe fn get_kernel_ptr(&self, index: usize) -> *mut u8 {
        (self.ppn.physical_to_kernel() + index).get_base_addr() as *mut u8
    }

    /// Gets mutable reference to frame memory as [T] at index.
    pub unsafe fn get_ref<'b, T>(&'b self, index: usize) -> &'b mut T {
        unsafe {
            let ptr = self.get_kernel_ptr(index);
            &mut *(ptr as *mut T)
        }
    }

    /// Gets mutable pointer to frame memory as [T] at index.
    pub unsafe fn get_ptr<T>(&self, index: usize) -> *mut T {
        unsafe {
            let ptr = self.get_kernel_ptr(index);
            ptr as *mut T
        }
    }

    /// Gets mutable slice of frame memory as [T].
    pub unsafe fn get_as_arr<'b, T>(&'b self, count: usize) -> &'b mut [T] {
        unsafe {
            let ptr = self.get_kernel_ptr(0) as *mut T;
            slice::from_raw_parts_mut(ptr, count)
        }
    }

    /// Gets [PageNum] of frame at index.
    pub fn get_ppn(&self, index: usize) -> PageNum {
        self.ppn + index
    }

    /// Gets total frame count.
    pub fn get_count(&self) -> usize {
        self.count
    }

    /// Iterates over all frames.
    pub fn iter(&self) -> impl Iterator<Item = PageNum> {
        (0..self.count).map(|x| self.get_ppn(x)).into_iter()
    }
}

/// Collection of non-contiguous frames.
pub struct ManagedFrames {
    frames: Vec<(usize, ManagedFrameRange)>,
    total: usize,
}

impl ManagedFrames {
    /// Creates new [ManagedFramesVec] from frame collection.
    pub fn new(frames: Vec<ManagedFrameRange>) -> ManagedFrames {
        let count = frames.iter().map(|f| f.count).sum();
        let mut frames_new = vec![];
        let mut offset = 0;
        for f in frames {
            let count = f.count;
            frames_new.push((offset, f));
            offset += count;
        }
        ManagedFrames {
            frames: frames_new,
            total: count,
        }
    }
    /// Finds frame containing index and its internal offset.
    unsafe fn get_frame(&self, index: usize) -> (&ManagedFrameRange, usize) {
        let len = self.frames.len();
        if len < 12 {
            // linear search
            for f in &self.frames {
                if f.0 + f.1.count > index {
                    return (&f.1, index - f.0);
                }
            }
            panic!(
                "Index out of bound when trying to get physical frame: {}.",
                index
            );
        } else {
            // binary search
            let idx = self
                .frames
                .binary_search_by(|val| {
                    if val.0 > index {
                        core::cmp::Ordering::Greater
                    } else if val.0 + val.1.count > index {
                        core::cmp::Ordering::Equal
                    } else {
                        core::cmp::Ordering::Less
                    }
                })
                .unwrap_or_else(|_| {
                    panic!(
                        "Index out of bound when trying to get physical frame: {}.",
                        index
                    );
                });
            let offset = index - self.frames[idx].0;
            (&self.frames[idx].1, offset)
        }
    }
    /// Gets total frame count.
    pub fn get_count(&self) -> usize {
        self.total
    }
    /// Gets mutable kernel address pointer for frame at index.
    pub unsafe fn get_kernel_ptr(&self, index: usize) -> *mut u8 {
        unsafe {
            let (frame, offset) = self.get_frame(index);
            frame.get_kernel_ptr(offset)
        }
    }

    /// Gets mutable reference to frame memory as [T] at index.
    pub unsafe fn get_ref<'a, T>(&'a self, index: usize) -> &'a mut T {
        unsafe {
            let (frame, offset) = self.get_frame(index);
            frame.get_ref(offset)
        }
    }

    /// Gets mutable pointer to frame memory as [T] at index.
    pub unsafe fn get_ptr<'a, T>(&'a self, index: usize) -> *mut T {
        unsafe {
            let (frame, offset) = self.get_frame(index);
            frame.get_ptr(offset)
        }
    }

    /// Gets [PageNum] of frame at index.
    pub fn get_ppn(&self, index: usize) -> PageNum {
        unsafe {
            let (frame, offset) = self.get_frame(index);
            frame.get_ppn(offset)
        }
    }
    /// Adds [ManagedFrames] to collection.
    pub fn add_frame(&mut self, frame: ManagedFrameRange) {
        self.total += frame.count;
        match self.frames.last() {
            Some(last) => {
                self.frames.push((last.0 + last.1.count, frame));
            }
            None => {
                self.frames.push((0, frame));
            }
        }
    }

    /// Iterates over all frames.
    pub fn iter(&self) -> impl Iterator<Item = PageNum> {
        (0..self.frames.len())
            .map(|x| &self.frames[x])
            .flat_map(|x| (0..x.1.count).map(|y| x.1.get_ppn(y)))
    }
}
// endregion

// region: Allocator

/// Default [FrameAllocator] implementation.
pub type DefaultFrameAllocator = buddy::BuddyFrameAllocator;

/// Global frame allocator instance.
pub static FRAME_ALLOC: LockedFrameAllocator<DefaultFrameAllocator> =
    LockedFrameAllocator::new(DefaultFrameAllocator::new());

/// Initializes frame allocator, filtering areas exceeding max physical address.
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
