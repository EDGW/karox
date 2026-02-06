use utils::define_struct;

use crate::{
    mm::{
        config::{KERNEL_STACK_PAGES, KERNEL_STACK_SIZE},
        frame::{FRAME_ALLOC, FrameAllocatorError, FrameRange},
    },
};

// region: KernelStack
define_struct!(copy_aligned, RawKernelStack, [u8; KERNEL_STACK_SIZE], 4096);
impl RawKernelStack {
    pub const fn new() -> RawKernelStack {
        RawKernelStack::from_const([0; KERNEL_STACK_SIZE])
    }
    pub fn get_stack_top(&self) -> *const u8 {
        unsafe { self.as_ptr().add(KERNEL_STACK_SIZE) }
    }
}

#[derive(Debug)]
pub struct KernelStack {
    frames: FrameRange,
}

impl KernelStack {
    /// Create a kernel stack and set the stack top
    pub fn new() -> Result<KernelStack, FrameAllocatorError> {
        let frames = FRAME_ALLOC.alloc_range_managed(KERNEL_STACK_PAGES)?;
        let res = KernelStack { frames: frames };
        Ok(res)
    }

    pub fn as_data_mut(&mut self) -> &mut RawKernelStack {
        unsafe {
            self.frames
                .as_ptr_mut::<RawKernelStack>(0)
                .as_mut()
                .unwrap()
        }
    }

    pub fn as_data_ref(&self) -> &RawKernelStack {
        unsafe { self.frames.as_ptr::<RawKernelStack>(0).as_ref().unwrap() }
    }

    pub fn get_stack_top(&self) -> usize {
        self.frames.start_ppn().physical_to_kernel().get_base_addr() + KERNEL_STACK_SIZE
    }
}
// endregion
