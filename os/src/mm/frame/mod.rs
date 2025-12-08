use alloc::vec::Vec;
use spin::Mutex;

use crate::{
    arch::mm::config::PAGE_WIDTH, define_struct_num, devices::device_info::MemoryAreaInfo,
    mm::frame::buddy::BuddyFrameAllocator,
};

pub mod buddy;

pub type DefaultFrameAllocator = BuddyFrameAllocator;

pub static FRAME_ALLOC: Mutex<DefaultFrameAllocator> = Mutex::new(DefaultFrameAllocator::new());

define_struct_num!(PhysicalPageNum, usize);
impl PhysicalPageNum {
    pub const fn from_addr(addr: usize) -> PhysicalPageNum {
        PhysicalPageNum(addr >> PAGE_WIDTH)
    }
}

pub trait FrameAllocator {
    fn init(&mut self, general_mem: &Vec<MemoryAreaInfo>);
    fn alloc(&mut self, count: usize) -> Option<PhysicalPageNum>;
    fn decalloc(&mut self, ppn: PhysicalPageNum, count: usize);
}

pub fn init_frame(general_mem: &Vec<MemoryAreaInfo>){
    FRAME_ALLOC.lock().init(general_mem);
}