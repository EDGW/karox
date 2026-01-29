//! This crete packed some references to the symbols defined in the linker script
#[allow(missing_docs)]

unsafe extern "C" {
    pub unsafe fn _skernel();
    pub unsafe fn _stext();
    pub unsafe fn _etext();
    pub unsafe fn _srodata();
    pub unsafe fn _erodata();
    pub unsafe fn _sdata();
    pub unsafe fn _edata();
    pub unsafe fn _sbss();
    pub unsafe fn _kbss();
    pub unsafe fn _ebss();
    pub unsafe fn _ekernel();
}

#[macro_export]
macro_rules! phys_addr_from_kernel {
    ($symbol: expr) => {{ ($symbol as *const u8 as usize) - crate::arch::KERNEL_OFFSET }};
}
