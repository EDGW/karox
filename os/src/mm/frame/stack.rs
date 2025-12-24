//! Stack-based Frame Allocator with Recyclation
#![allow(unused)]

use alloc::{vec, vec::Vec};

use crate::{
    arch::mm::{config::Paging, paging::PageNum},
    mm::{PagingMode, frame::FrameAllocator},
    utils::range::Range,
};
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
            start: general_mem.start / Paging::PAGE_SIZE,
            length: general_mem.length / Paging::PAGE_SIZE,
        });
    }

    unsafe fn try_alloc(&mut self, count: usize) -> Option<crate::arch::mm::paging::PageNum> {
        if count > 1 {
            if self.allow_contiguous {
                for r in &mut self.free {
                    if r.length > count {
                        r.length -= count;
                        return Some(PageNum::from_value(r.start + r.length));
                    }
                }
                None
            } else {
                None
            }
        } else {
            if let Some(ppn) = self.recycled.pop() {
                Some(PageNum::from_value(ppn))
            } else {
                for r in &mut self.free {
                    if r.length > 1 {
                        r.length -= 1;
                        return Some(PageNum::from_value(r.start + r.length));
                    }
                }
                None
            }
        }
    }

    unsafe fn decalloc(&mut self, ppn: crate::arch::mm::paging::PageNum, count: usize) {
        for i in ppn.get_value()..(ppn.get_value() + count) {
            self.recycled.push(i);
        }
    }
}
