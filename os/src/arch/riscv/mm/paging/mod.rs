//! RISC-V SV-Based Paging
//!
//! This module provides the implementation of RISC-V SV-based paging, including the representation
//! of page numbers, page table entries, and page tables. It also includes utilities for creating
//! and managing page tables during early boot steps.

use crate::{
    arch::mm::config::Paging, define_struct_copy_aligned, define_struct_num, mm::PagingMode,
};
use bitflags::bitflags;

pub mod sv39;

// Represents a page number.
//
// Provides methods to convert between addresses and page numbers, as well as utilities for
// kernel-to-physical and physical-to-kernel address translations.
define_struct_num!(PageNum, usize);
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

bitflags! {
    /// Flags for a page table entry.
    pub struct PageTableFlags: u8{
        /// Indicates a valid entry. Without this flag, the entry is not recognized.
        const VALID     = 0b1;

        /// Indicates a page directory entry in tables that contains non-leaf pages.
        const DIR       = 0b000_0;
        /// Read-only permission, typically used for read-only data sections.
        const R         = 0b001_0;
        /// Write-only permission.
        const W         = 0b010_0;
        /// Execute-only permission.
        const X         = 0b100_0;
        /// Read-write permission, typically used for data sections.
        const RW        = Self::R.bits | Self::W.bits;
        /// Read-execute permission, typically used for text sections.
        const RX        = Self::R.bits | Self::X.bits;
        /// Full permissions (read, write, execute).
        const RWX       = Self::R.bits | Self::W.bits | Self::X.bits;

        /// Indicates that the page can be accessed in user mode.
        const USER      = 0b000_1_000_0;
        /// Indicates that the page is shared among all page tables.
        ///
        /// A page entry with the `GLOBAL` flag will be kept in the TLB cache. Changing the mappings
        /// for a global entry may lead to unexpected consequences.
        const GLOBAL    = 0b000_0_000_0;
        /// Accessed bit, indicating that the page has been accessed.
        const ACCESSED  = 0b010_0_000_0;
        /// Dirty bit, indicating that the page has been written to.
        const DIRTY     = 0b100_0_000_0;
    }
}

// Represents a page table entry.
//
// Provides methods to create and manipulate page table entries, including setting flags and
// creating invalid entries.
define_struct_num!(PageTableEntry, usize);
impl PageTableEntry {
    const FLAG_BITS_MASK: usize = 0xff;

    /// Retrieves the flags of the page table entry.
    #[inline(always)]
    pub const fn get_flags(&self) -> PageTableFlags {
        PageTableFlags {
            bits: (self.get_value() & Self::FLAG_BITS_MASK) as u8,
        }
    }

    /// Creates a page table entry from a page number and flags.
    #[inline(always)]
    pub const fn create(ppn: PageNum, flags: PageTableFlags) -> PageTableEntry {
        let mut p = ppn.get_value() << 10;
        p = p | (flags.bits as usize);
        PageTableEntry::from_value(p)
    }

    /// Creates an invalid page table entry.
    ///
    /// The `valid` bit of the flags is guaranteed to be 0.
    #[inline(always)]
    pub const fn create_invalid() -> PageTableEntry {
        PageTableEntry::from_value(0)
    }
}

// Represents a page table.
define_struct_copy_aligned!(
    PageTableValue,
    [PageTableEntry; Paging::PTABLE_ENTRY_COUNT],
    4096
);
impl PageTableValue {
    /// Creates a new page table filled with invalid entries.
    pub const fn new_empty() -> PageTableValue {
        Self::from_value([PageTableEntry::create_invalid(); Paging::PTABLE_ENTRY_COUNT])
    }
}

/// The page table used during early boot steps.
///
/// This table is abandoned after [`Paging::init()`] is called.
#[allow(long_running_const_eval)]
pub static BOOT_PTABLE: PageTableValue = create_kernel_linear_ptable();

/// Fills a range of entries in a page table with mappings to specific address spaces.
///
/// * `table` - The page table to fill.
/// * `starting_index` - The starting index in the table.
/// * `starting_ppn` - The starting page number.
/// * `count` - The number of entries to fill.
/// * `page_size` - The size of each page.
/// * `flags` - The flags for the entries.
#[inline(always)]
pub const fn fill_linear_ptable(
    table: &mut PageTableValue,
    starting_index: usize,
    starting_ppn: PageNum,
    count: usize,
    page_size: usize,
    flags: PageTableFlags,
) {
    let mut i = 0;
    while i < count {
        // 1 GiB large page, 0xcf = V_RWX_AD
        table.0[starting_index + i] = PageTableEntry::create(
            PageNum::from_addr(
                page_size * i + starting_ppn.get_base_addr(),
                // PageNum + PageNum is not available in const fns
            ),
            flags,
        );
        i += 1;
    }
}

/// Creates the kernel linear page table used during early boot steps.
#[inline(always)]
pub const fn create_kernel_linear_ptable() -> PageTableValue {
    let mut res = PageTableValue::new_empty();
    let flags = PageTableFlags { bits: 0xcf }; // Hard-coded bits for const functions
    let mid = Paging::PTABLE_ENTRY_COUNT / 2;
    fill_linear_ptable(
        &mut res,
        0,
        PageNum::from_value(0),
        mid,
        Paging::PG_SIZE_L0,
        flags,
    );
    fill_linear_ptable(
        &mut res,
        mid,
        PageNum::from_value(0),
        Paging::PTABLE_ENTRY_COUNT - mid,
        Paging::PG_SIZE_L0,
        flags,
    );
    res
}
