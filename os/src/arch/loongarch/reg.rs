//! This module defines the offset values for loongarch CSRs

use crate::{arch::{CombinablePriority, mm::MemAccessType}, define_struct_num};

/// Current Mode CSR
pub const CR_CRMD: u16 = 0x0;

/// Previous Mode CSR
pub const CR_PRMD: u16 = 0x1;

/// CPUID CSR
pub const CR_CPUID: u16 = 0x20;

/// Direct Mapped Window 0 CSR
pub const CR_DMW0: u16 = 0x180;

/// Direct Mapped Window 1 CSR
pub const CR_DMW1: u16 = 0x181;

/// Direct Mapped Window 2 CSR
pub const CR_DMW2: u16 = 0x182;


define_struct!(num,CrDMWValue, usize);
impl CrDMWValue {
    const VSEG_FILTER: usize = 0xf000_0000_0000_0000;

    /// Create a DMW from the given flags and addr.
    ///
    /// Only the top 4 bits of the address are available for configuration, and other bits are masked.
    pub const fn create(plv_flags: CombinablePriority, mat: MemAccessType, addr: usize) -> CrDMWValue {
        let mut res = addr & Self::VSEG_FILTER;
        res |= plv_flags.bits() as usize;
        res |= (mat.bits() as usize) >> 4;
        CrDMWValue(res)
    }
}
