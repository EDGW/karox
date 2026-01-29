/// **You should change the linker scripts arguments defined in link_flags.json if you change the paging mode.**
use riscv::register::satp;

// SV39
pub const SV_LEN: usize = 39;
pub const SATP_MODE: satp::Mode = satp::Mode::Sv39;
pub const PTABLE_MAX_LEVEL: usize = 2;

// // SV48
// pub const SV_LEN:usize = 48;
// pub const SATP_MODE: satp::Mode = satp::Mode::Sv48;
// pub const PTABLE_MAX_LEVEL: usize = 3;

// // SV57
// pub const SV_LEN:usize = 57;
// pub const SATP_MODE: satp::Mode = satp::Mode::Sv57;
// pub const PTABLE_MAX_LEVEL: usize = 4;

/// Max user space addr; Size of user space and kernel space.
pub const MAX_USPACE_ADDR: usize = 1 << (SV_LEN - 1);

pub const KERNEL_OFFSET: usize = usize::MAX - MAX_USPACE_ADDR + 1;

/// The lower half of the kernel address space is used for direct-offset mapping;
/// therefore the maximum physical address is limited to half of the kernel address space size.
pub const MAX_PHYS_ADDR: usize = MAX_USPACE_ADDR / 2;

pub const MAX_ASID: usize = (1 << 16) - 1;
pub const KERNEL_ASID: usize = 0;
