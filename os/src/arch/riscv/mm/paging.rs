//! This module contains paging functions and strategies for riscv-64

use bitflags::bitflags;

use crate::{
    arch::mm::config::{PAGE_WIDTH, PTABLE_ENTRY_COUNT},
    define_struct_copy, define_struct_copy_aligned,
};

// A 64-bit page table entry wrapper.
define_struct_copy!(PageTableEntry, u64);

// Physical page number (PPN) wrapper.
define_struct_copy!(PhysicalPageNum, u64);

impl PhysicalPageNum {
    /// Create a `PhysicalPageNum` from a physical byte address.
    ///
    /// Shifts the address right by `PAGE_WIDTH` (page size exponent) to obtain
    /// the page number.
    pub const fn from_addr(addr: usize) -> PhysicalPageNum {
        PhysicalPageNum::from_value((addr >> PAGE_WIDTH) as u64)
    }
}

impl PageTableEntry {
    const FLAG_MASK: u64 = 0xff;

    /// Get the flags from the PTE
    #[inline(always)]
    pub fn get_flags(self) -> u8 {
        return (*self & Self::FLAG_MASK) as u8;
    }
    /// Create a PTE from a given PPN and the flags..
    pub const fn create(ppn: PhysicalPageNum, flags: u8) -> PageTableEntry {
        let mut p = ppn.0 << 10;
        p = p | (flags as u64);
        PageTableEntry(p)
    }
}

// A page-aligned page table type.
//
// The page table is an array of `PageTableEntry` with length
// `PTABLE_ENTRY_COUNT`. The alignment (4096) ensures the table starts at a
// page boundary which is required by the hardware for root page tables.
define_struct_copy_aligned!(PageTable, [PageTableEntry; PTABLE_ENTRY_COUNT], 4096);

/// Initial boot page table.
///
/// Constructed at compile time to provide a linear mapping of the kernel in high address.
#[allow(long_running_const_eval)]
pub static BOOT_PTABLE: PageTable = create_full_ptable();

/// Create a linear mapping to the high half kernel address
#[inline(always)]
pub const fn create_full_ptable() -> PageTable {
    let mut ptable_ls = [PageTableEntry(0); PTABLE_ENTRY_COUNT];
    let mut i = 0;
    while 2 * i < PTABLE_ENTRY_COUNT {
        // 1 GiB large page, 0xcf = V_RWX_AD
        ptable_ls[i] = PageTableEntry::create(PhysicalPageNum(0x40000 * i as u64), 0xcf);
        ptable_ls[256 + i] = PageTableEntry::create(PhysicalPageNum(0x40000 * i as u64), 0xcf);
        i += 1;
    }
    PageTable::from_value(ptable_ls)
}

bitflags! {
    /// The Paging Modes, used in `satp` register
    pub struct CrSatpModes : u8{
        /// Non-Paging
        const BARE  = 0;
        /// SV39 Paging Strategy
        const SV39  = 8;
    }
}

// Pack a `satp` value from its components.
//
// `satp` register layout (as used here):
// - bits 60..63: MODE
// - bits 44..59: ASID
// - bits 0..43 : PPN
define_struct_copy!(CrSatpValue, u64);

impl CrSatpValue {
    /// Create a new `satp` register value
    pub const fn create(mode: CrSatpModes, asid: u16, ppn: PhysicalPageNum) -> CrSatpValue {
        CrSatpValue(((mode.bits as u64) << 60) | ((asid as u64) << 44) | ppn.0)
    }
}
