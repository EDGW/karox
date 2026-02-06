use core::fmt::Debug;

use utils::{impl_basic, impl_number};

use crate::{
    arch::{KERNEL_OFFSET, PAGE_WIDTH},
};

// region: PageNum
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct PageNum {
    inner: usize,
}
impl_basic!(PageNum, usize);
impl_number!(PageNum, usize);

impl Debug for PageNum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.inner))
    }
}

impl PageNum {
    pub const fn from_addr(addr: usize) -> PageNum {
        PageNum::from_const(addr >> PAGE_WIDTH)
    }

    pub const fn get_base_addr(&self) -> usize {
        self.into_const() << PAGE_WIDTH
    }

    /// Remove the kernel space bits, converting it to physical page num(ppn).
    pub const fn kernel_to_physical(&self) -> PageNum {
        PageNum::from_const(self.into_const() & (!KERNEL_OFFSET >> PAGE_WIDTH))
    }

    /// Add the kernel space bits, converting it to virtual page num(vpn).
    pub const fn physical_to_kernel(&self) -> PageNum {
        PageNum::from_const(self.into_const() | (KERNEL_OFFSET >> PAGE_WIDTH))
    }
}

// endregion
