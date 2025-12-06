use bitflags::bitflags;

use crate::{
    arch::mm::config::{PAGE_WIDTH, PTABLE_ENTRY_COUNT},
    define_struct_copy, define_struct_copy_aligned,
};

define_struct_copy!(PageTableEntry, u64);
define_struct_copy!(PhysicalPageNum, u64);

impl PhysicalPageNum {
    pub const fn from_addr(addr: usize) -> PhysicalPageNum {
        PhysicalPageNum::from_value((addr >> PAGE_WIDTH) as u64)
    }
}

impl PageTableEntry {
    const FLAG_MASK: u64 = 0xff;
    #[inline(always)]
    pub fn get_flags(self) -> u8 {
        return (*self & Self::FLAG_MASK) as u8;
    }
    pub const fn create(ppn: PhysicalPageNum, flags: u8) -> PageTableEntry {
        let mut p = ppn.0 << 10;
        p = p | (flags as u64);
        PageTableEntry(p)
    }
}

define_struct_copy_aligned!(PageTable, [PageTableEntry; PTABLE_ENTRY_COUNT], 4096);

#[allow(long_running_const_eval)]
pub static BOOT_PTABLE: PageTable = create_full_ptable();

#[inline(always)]
pub const fn create_full_ptable() -> PageTable {
    let mut ptable_ls = [PageTableEntry(0); PTABLE_ENTRY_COUNT];
    let mut i = 0;
    while 2 * i < PTABLE_ENTRY_COUNT {
        ptable_ls[i] = PageTableEntry::create(PhysicalPageNum(0x40000 * i as u64), 0xcf);
        ptable_ls[256 + i] = PageTableEntry::create(PhysicalPageNum(0x40000 * i as u64), 0xcf);
        i += 1;
    }
    PageTable::from_value(ptable_ls)
}

bitflags! {
    pub struct CrSatpModes : u8{
        const BARE  = 0;
        const SV39  = 8;
    }
}

define_struct_copy!(CrSatpValue, u64);
impl CrSatpValue {
    pub const fn create(mode: CrSatpModes, asid: u16, ppn: PhysicalPageNum) -> CrSatpValue {
        CrSatpValue(((mode.bits as u64) << 60) | ((asid as u64) << 44) | ppn.0)
    }
}
