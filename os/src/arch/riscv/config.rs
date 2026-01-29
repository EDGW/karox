use crate::arch::mm::sv;

pub const PAGE_WIDTH: usize = 12;

pub const PTABLE_MAX_LEVEL: usize = sv::PTABLE_MAX_LEVEL;

pub const KERNEL_OFFSET: usize = sv::KERNEL_OFFSET;

pub const MAX_USPACE_ADDR: usize = sv::MAX_USPACE_ADDR;

pub const MAX_PHYS_ADDR: usize = sv::MAX_PHYS_ADDR; // 128GiB

pub const MAX_ASID: usize = sv::MAX_ASID;

pub const KERNEL_ASID: usize = sv::KERNEL_ASID;

pub const MAX_HARTS: usize = 16;
