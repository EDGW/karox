
use crate::{arch::{PrvLevelBits, mm::MemAccessType}, define_struct_copy};

define_struct_copy!(CrDMWValue,u64);

impl CrDMWValue{
    const VSEG_FILTER:u64   =   0xf000_0000_0000_0000;
    pub const fn create(plv_flags: PrvLevelBits, mat: MemAccessType, addr: usize) -> CrDMWValue{
        let mut res = (addr as u64) & Self::VSEG_FILTER;
        res |= plv_flags.bits as u64;
        res |= (mat.bits as u64)>>4;
        CrDMWValue(res)
    }
}