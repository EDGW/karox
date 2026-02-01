//! Buddy Frame Allocator
//!
//! This module packed a buddy frame allocator.

#![allow(unused)]

use crate::{arch::mm::PageNum, devices::device_info::MemoryAreaInfo, mm::frame::FrameAllocator};

/// Maximum order for the buddy system.
pub const MAX_ORDER: usize = 32;

/// Buddy frame allocator implementation.
#[allow(unused)]
pub struct BuddyFrameAllocator {
    inner: buddy_system_allocator::FrameAllocator<MAX_ORDER>,
}

impl BuddyFrameAllocator {
    /// Creates a new buddy frame allocator.
    pub const fn new() -> Self {
        BuddyFrameAllocator {
            inner: buddy_system_allocator::FrameAllocator::new(),
        }
    }
}

impl FrameAllocator for BuddyFrameAllocator {
    unsafe fn alloc(&mut self, count: usize) -> Option<PageNum> {
        self.inner.alloc(count).map(PageNum::from)
    }
    unsafe fn dealloc(&mut self, ppn: PageNum, count: usize) {
        self.inner.dealloc(ppn.into(), count);
    }
    fn add_frame(&mut self, general_mem: MemoryAreaInfo) {
        let start = PageNum::from_addr(general_mem.start);
        let end = PageNum::from_addr(general_mem.end);
        self.inner.add_frame(start.into(), end.into());
    }
}
