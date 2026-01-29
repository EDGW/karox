use crate::{
    arch::mm::PageNum,
    mm::frame::{FRAME_ALLOC, FrameAllocator},
};
use alloc::vec::Vec;
use core::fmt::Debug;

pub struct Frame {
    ppn: PageNum,
}
impl Frame {
    /// Create a managed frame from a physical page number.
    ///
    /// The function is marked as **unsafe** because
    /// **trying to deallocate a ghost frame will lead to undefined behavior.**
    pub unsafe fn new(ppn: PageNum) -> Self {
        Frame { ppn }
    }

    /// Get the physical page number of the frame.
    pub fn ppn(&self) -> PageNum {
        self.ppn
    }

    /// Get the kernel virtual page number of the frame.
    pub fn kvpn(&self) -> PageNum {
        self.ppn.physical_to_kernel()
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.kvpn().get_base_addr() as *const T
    }

    pub fn as_ptr_mut<T>(&self) -> *mut T {
        self.kvpn().get_base_addr() as *mut T
    }
}

impl Debug for Frame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.ppn))
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            FRAME_ALLOC.lock().dealloc(self.ppn, 1);
        }
    }
}

/// A set of managed frames, strong-ordered, once initialized, and not promised to be contiguous.
pub struct FrameSet {
    frames: Vec<Frame>,
}

impl Debug for FrameSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.frames.fmt(f)
    }
}

impl FrameSet {
    /// Create from physical page numbers.
    ///
    /// The function is marked as **unsafe** because **it actually creates [Frame] instances, and
    /// trying to deallocate a ghost frame will lead to undefined behavior.**
    pub unsafe fn from_pn<T: IntoIterator<Item = PageNum>>(ppns: T) -> Self {
        let frames = ppns.into_iter().map(|x| unsafe { Frame::new(x) }).collect();
        FrameSet { frames }
    }

    pub fn new(frames: Vec<Frame>) -> Self {
        FrameSet { frames }
    }

    /// Get a specific frame.
    pub fn get_frame(&self, index: usize) -> &Frame {
        &self.frames[index]
    }
}

pub struct FrameRange {
    start: PageNum,
    count: usize,
}

impl Debug for FrameRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "[{:?},{:?})",
            self.start,
            self.start + self.count
        ))
    }
}

impl FrameRange {
    /// Create a managed frame range from a starting physical page number and its length.
    ///
    /// The function is marked as **unsafe** because
    /// **trying to deallocate a ghost frame will lead to undefined behavior.**
    pub unsafe fn new(start: PageNum, count: usize) -> Self {
        FrameRange { start, count }
    }

    /// Get the starting physical page number.
    pub fn start_ppn(&self) -> PageNum {
        self.start
    }

    pub fn start_kvpn(&self) -> PageNum{
        self.start.physical_to_kernel()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn get_ppn(&self, index: usize) -> PageNum {
        debug_assert!(index < self.count);
        self.start + index
    }

    pub fn get_kvpn(&self, index: usize) -> PageNum {
        debug_assert!(index < self.count);
        (self.start + index).physical_to_kernel()
    }

    pub fn as_ptr<T>(&self, index: usize) -> *const T {
        self.get_kvpn(index).get_base_addr() as *const T
    }

    pub fn as_ptr_mut<T>(&self, index: usize) -> *mut T {
        self.get_kvpn(index).get_base_addr() as *mut T
    }
}
impl Drop for FrameRange {
    fn drop(&mut self) {
        unsafe {
            FRAME_ALLOC.lock().dealloc(self.start, self.count);
        }
    }
}
