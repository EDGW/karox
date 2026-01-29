//! Stack-based Frame Allocator with Recyclation
#![allow(unused)]

use crate::{
    arch::mm::PageNum,
    mm::{config::PAGE_SIZE, frame::FrameAllocator},
};
use alloc::{vec, vec::Vec};
use core::ops::Range;
/// Stack-based frame allocator using free ranges and recycled pages.
pub struct StackFrameAllocator {
    /// Available memory ranges.
    free: Vec<Range<usize>>,
    /// Recycled page numbers for reallocation.
    recycled: Vec<usize>,
    /// Whether contiguous allocation is allowed.
    allow_contiguous: bool,
}
impl StackFrameAllocator {
    /// Creates new [StackFrameAllocator] with contiguity configuration.
    pub const fn new(allow_contiguous: bool) -> StackFrameAllocator {
        StackFrameAllocator {
            free: vec![],
            recycled: vec![],
            allow_contiguous,
        }
    }
}
/// [FrameAllocator] trait implementation for stack-based allocation.
impl FrameAllocator for StackFrameAllocator {
    fn add_frame(&mut self, general_mem: crate::devices::device_info::MemoryAreaInfo) {
        self.free.push(Range {
            start: general_mem.start / PAGE_SIZE,
            end: general_mem.end / PAGE_SIZE,
        });
    }

    unsafe fn alloc(&mut self, count: usize) -> Option<PageNum> {
        if count > 1 {
            if self.allow_contiguous {
                for r in &mut self.free {
                    if r.len() > count {
                        r.end -= count;
                        return Some(PageNum::from(r.end));
                    }
                }
                None
            } else {
                None
            }
        } else {
            if let Some(ppn) = self.recycled.pop() {
                Some(PageNum::from(ppn))
            } else {
                for r in &mut self.free {
                    if r.len() > 1 {
                        r.end -= 1;
                        return Some(PageNum::from(r.end));
                    }
                }
                None
            }
        }
    }

    unsafe fn dealloc(&mut self, ppn: PageNum, count: usize) {
        let st: usize = ppn.into();
        let ed = st + count;
        for i in st..ed {
            self.recycled.push(i);
        }
    }
}
