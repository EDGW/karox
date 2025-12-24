//! Buddy Frame Allocator
//! 
//! This module packed a buddy frame allocator.

use crate::{
    arch::mm::paging::PageNum, devices::device_info::MemoryAreaInfo, mm::frame::FrameAllocator,
};

/// Maximum order for the buddy system.
pub const MAX_ORDER: usize = 32;

/// Buddy frame allocator implementation.
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
    unsafe fn try_alloc(&mut self, count: usize) -> Option<PageNum> {
        self.inner.alloc(count).map(PageNum::from_value)
    }
    unsafe fn decalloc(&mut self, ppn: PageNum, count: usize) {
        self.inner.dealloc(ppn.get_value(), count);
    }
    fn add_frame(&mut self, general_mem: MemoryAreaInfo) {
        let start = PageNum::from_addr(general_mem.start);
        let end = PageNum::from_addr(general_mem.start + general_mem.length);
        self.inner.add_frame(start.get_value(), end.get_value());
    }
    fn init(&mut self) {}
}
