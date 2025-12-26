//! Loongarch Paging

use crate::{arch::mm::config::Paging,  mm::PagingMode};

// Represents a page number.
//
// Provides methods to convert between addresses and page numbers, as well as utilities for
// kernel-to-physical and physical-to-kernel address translations.
define_struct!(num,PageNum, usize);
impl PageNum {
    /// Creates a [PageNum] from an address.
    #[inline(always)]
    pub const fn from_addr(addr: usize) -> PageNum {
        PageNum::from_value(addr >> Paging::PAGE_WIDTH)
    }

    /// Returns the base address of the page.
    #[inline(always)]
    pub const fn get_base_addr(&self) -> usize {
        self.get_value() << Paging::PAGE_WIDTH
    }

    /// Converts a kernel virtual address to a physical address.
    #[inline(always)]
    pub const fn kernel_to_physical(&self) -> PageNum {
        PageNum::from_value(self.get_value() & (!Paging::KERNEL_OFFSET >> Paging::PAGE_WIDTH))
    }

    /// Converts a physical address to a kernel virtual address.
    #[inline(always)]
    pub const fn physical_to_kernel(&self) -> PageNum {
        PageNum::from_value(self.get_value() | (Paging::KERNEL_OFFSET >> Paging::PAGE_WIDTH))
    }
}

const GB: usize = 1024 * 1024 * 1024;
const MB: usize = 1024 * 1024;
const KB: usize = 1024;

pub struct LAPaging;
impl LAPaging{
    /// The number of the entries in a page table
    pub const PTABLE_ENTRY_COUNT: usize = 512;
}
impl PagingMode for LAPaging{
    const PAGE_SIZE: usize = 4 * KB;
    const PAGE_WIDTH: usize = 12;
    const KERNEL_OFFSET: usize = 0x9000_0000_0000_0000;
    const MMIO_OFFSET: usize = 0x8000_0000_0000_0000;
    const MAX_PHYSICAL_ADDR: usize = 512 * GB;
    fn init() {
        
    }
}