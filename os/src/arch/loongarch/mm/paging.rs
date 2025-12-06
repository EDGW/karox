//! This module contains paging functions and strategies for loongarch64
use crate::{
    arch::{PrvLevelBits, mm::MemAccessType},
    define_struct_copy,
};

define_struct_copy!(CrDMWValue, u64);

impl CrDMWValue {
    const VSEG_FILTER: u64 = 0xf000_0000_0000_0000;

    /// Create a DMW from the given flags and addr.
    ///
    /// Only the top 4 bits of the address are available for configuration, and other bits are masked.
    pub const fn create(plv_flags: PrvLevelBits, mat: MemAccessType, addr: usize) -> CrDMWValue {
        let mut res = (addr as u64) & Self::VSEG_FILTER;
        res |= plv_flags.bits as u64;
        res |= (mat.bits as u64) >> 4;
        CrDMWValue(res)
    }
}
