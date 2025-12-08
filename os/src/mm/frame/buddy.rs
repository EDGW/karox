use alloc::vec::Vec;

use crate::{
    devices::device_info::MemoryAreaInfo,
    mm::frame::{FrameAllocator, PhysicalPageNum},
};

pub type BuddyFrameAllocator = buddy_system_allocator::FrameAllocator<MAX_ORDER>;

pub const MAX_ORDER: usize = 32;

impl FrameAllocator for BuddyFrameAllocator {
    fn alloc(&mut self, count: usize) -> Option<PhysicalPageNum> {
        self.alloc(count).map(PhysicalPageNum::from_value)
    }
    fn decalloc(&mut self, ppn: PhysicalPageNum, count: usize) {
        self.dealloc(ppn.get_value(), count);
    }
    fn init(&mut self, general_mem: &Vec<MemoryAreaInfo>) {
        for mem_area in general_mem {
            let start = PhysicalPageNum::from_addr(mem_area.start);
            let end = PhysicalPageNum::from_addr(mem_area.start + mem_area.length);
            self.add_frame(start.get_value(), end.get_value());
        }
    }
}
