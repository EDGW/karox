use crate::{
    arch::{
        KERNEL_ASID, KERNEL_OFFSET, MAX_USPACE_ADDR, PAGE_WIDTH,
        mm::{PageNum, sv::SATP_MODE},
        symbols::{_ebss, _edata, _erodata, _etext, _sbss, _sdata, _srodata, _stext},
    },
    debug_ex,
    mm::{
        config::PTABLE_ENTRY_COUNT,
        frame::{FRAME_ALLOC, Frame, FrameAllocatorError},
        paging::{PageDirTrait, PageTable, PagingError},
        space::MemSpace,
    },
    phys_addr_from_symbol,
};
use alloc::boxed::Box;
use bitflags::bitflags;
use core::{
    array,
    fmt::Debug,
    ops::{Deref, Range},
    sync::atomic::{AtomicUsize, Ordering},
};
use lazy_static::lazy_static;
use riscv::register::satp;
use utils::impl_basic;

// region: PageTableFlags
bitflags! {
    /// Flags for a page table entry.
    pub struct PageTableFlags: u8{
        /// No Flags
        const NUL       = 0b0;

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
        const GLOBAL    = 0b001_0_000_0;
        /// Accessed bit, indicating that the page has been accessed.
        const ACCESSED  = 0b010_0_000_0;
        /// Dirty bit, indicating that the page has been written to.
        const DIRTY     = 0b100_0_000_0;

        /// Predefined value for page table dir entry. Other bits a set to 0;
        const PREDEFINED_DIR = Self::VALID.bits | Self::DIR.bits;

        /// Predefined value for boot entry. Representing huge pages;
        const PREDEFINED_BOOT = Self::VALID.bits | Self::RWX.bits | Self::ACCESSED.bits | Self::DIRTY.bits;
    }
}
// endregion

// region: PageTableEntry
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    inner: usize,
}

impl_basic!(PageTableEntry, usize);

/// Sv39/48/57 Page Table Entry Format:
/// 63     54 53   10 9        8 7        0
/// +--------+-------+----------+---------+
/// | RSV(0) |  PPN  | RSV(IGN) |  FLAGS  |
/// +--------+-------+----------+---------+
impl PageTableEntry {
    const FLAGS_MASK: usize = (1 << 8) - 1;
    const PPN_MASK: usize = (1 << (54 - 10)) - 1;

    pub const fn get_flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate((self.into_const() & Self::FLAGS_MASK) as u8)
    }

    pub const fn get_ppn(&self) -> PageNum {
        PageNum::from_const((self.into_const() >> 10) & Self::PPN_MASK)
    }

    pub const fn is_valid(&self) -> bool {
        self.get_flags().contains(PageTableFlags::VALID)
    }

    pub const fn is_dir(&self) -> bool {
        !self.get_flags().intersects(PageTableFlags::RWX)
    }

    /// Creates a page table entry from a physical page number and flags.
    pub const fn create(ppn: PageNum, flags: PageTableFlags) -> PageTableEntry {
        let mut p = (ppn.into_const() & Self::PPN_MASK) << 10;
        p = p | (flags.bits as usize);
        PageTableEntry::from_const(p)
    }

    /// Creates an invalid page table entry.
    ///
    /// The `valid` bit of the flags is guaranteed to be 0.
    #[inline(always)]
    pub const fn create_invalid() -> PageTableEntry {
        PageTableEntry::from_const(0)
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "Entry({:#x} | {:?})",
            self.get_ppn().into_const(),
            self.get_flags()
        ))
    }
}

// endregion

// region: RawPageDir
// Represents a page table that is thread-safe.
#[repr(C, align(4096))]
#[derive(Debug)]
pub struct RawPageDir {
    inner: [AtomicUsize; PTABLE_ENTRY_COUNT],
}

impl RawPageDir {
    const INVALID_ENTRY: AtomicUsize = AtomicUsize::new(0);
    /// Creates a new page table filled with invalid entries.
    pub const fn new_empty() -> RawPageDir {
        Self {
            inner: [Self::INVALID_ENTRY; PTABLE_ENTRY_COUNT],
        }
    }
    pub const fn from_const(value: [PageTableEntry; PTABLE_ENTRY_COUNT]) -> RawPageDir {
        let mut i = 0;
        let mut res = [Self::INVALID_ENTRY; PTABLE_ENTRY_COUNT];
        while i < PTABLE_ENTRY_COUNT {
            res[i] = AtomicUsize::new(value[i].into_const());
            i += 1;
        }
        RawPageDir { inner: res }
    }
    /// Safely update a page table entry. The operation is **atomic**.
    pub fn set_value(&self, index: usize, value: PageTableEntry) {
        self.inner[index].store(value.into(), Ordering::Relaxed);
    }
    /// Safely read a page table entry. The operation is **atomic**.
    pub fn get_value(&self, index: usize) -> PageTableEntry {
        PageTableEntry::from(self.inner[index].load(Ordering::Relaxed))
    }
}
// endregion

// region: PageDir
#[derive(Debug)]
pub struct PageDir {
    frame: Frame,
    subdirs: [Option<Box<PageDir>>; PTABLE_ENTRY_COUNT],
}

impl PageDir {
    pub unsafe fn new(frame: Frame) -> PageDir {
        PageDir {
            frame,
            subdirs: array::from_fn(|_| None),
        }
    }
    pub unsafe fn new_empty(frame: Frame) -> PageDir {
        let mut res = unsafe { Self::new(frame) };
        unsafe {
            res.clear();
        }
        res
    }
    unsafe fn as_data_mut(&mut self) -> &mut RawPageDir {
        unsafe { self.frame.as_ptr_mut::<RawPageDir>().as_mut().unwrap() }
    }

    unsafe fn as_data_ref(&self) -> &RawPageDir {
        unsafe { self.frame.as_ptr::<RawPageDir>().as_ref().unwrap() }
    }

    pub unsafe fn clear(&mut self) {
        let raw = unsafe { self.as_data_mut() };
        for i in 0..PTABLE_ENTRY_COUNT {
            raw.set_value(i, PageTableEntry::create_invalid());
        }
    }
    pub fn ppn(&self) -> PageNum {
        self.frame.ppn()
    }
}

impl PageDirTrait for PageDir {
    const LEVEL_WIDTH: usize = 9;
    const PPN_ALIGNED: bool = true;

    unsafe fn fill(
        &mut self,
        index: usize,
        ppn: PageNum,
        count: usize,
        step: usize,
        flags: PageTableFlags,
    ) {
        let raw = unsafe { self.as_data_mut() };
        for i in 0..count {
            debug_assert!(
                !(raw.get_value(index + i) as PageTableEntry).is_valid(),
                "Try to override page table entries that is valid."
            );
            raw.set_value(index + i, PageTableEntry::create(ppn + i * step, flags));
        }
    }

    fn get_or_expand(
        &mut self,
        index: usize,
        expand_step: usize,
    ) -> Result<&mut Self, FrameAllocatorError> {
        #[cfg(debug_assertions)]
        {
            // Debug Check
            let subdir_is_none = if let None = self.subdirs[index] {
                true
            } else {
                false
            };
            let raw = unsafe { self.as_data_mut() };
            debug_assert!(
                raw.get_value(index).is_dir() == (!subdir_is_none),
                "Inconsistent record in page table entries at {index}, value is {:?} while subdir {}",
                raw.get_value(index),
                subdir_is_none
            );
        }
        if let None = self.subdirs[index] {
            // Create New
            let mut dir = unsafe { PageDir::new_empty(FRAME_ALLOC.alloc_managed()?) };
            let raw = unsafe { self.as_data_mut() };
            let entry = raw.get_value(index);
            if entry.is_valid() {
                unsafe {
                    dir.fill(
                        0,
                        entry.get_ppn(),
                        PTABLE_ENTRY_COUNT,
                        expand_step,
                        entry.get_flags(),
                    );
                }
            }
            raw.set_value(
                index,
                PageTableEntry::create(dir.ppn(), PageTableFlags::PREDEFINED_DIR),
            );
            self.subdirs[index] = Some(Box::new(dir));
        }
        Ok(self.subdirs[index].as_mut().unwrap())
    }

    fn unfill(&mut self, index: usize, count: usize) {
        let raw = unsafe { self.as_data_mut() };
        for i in 0..count {
            raw.set_value(index + i, PageTableEntry::create_invalid());
        }
        for i in 0..count {
            self.subdirs[index + i] = None;
        }
    }

    fn get_or_none(&self, index: usize) -> Option<&Self> {
        let r = &self.subdirs[index];
        if let Some(res) = r {
            Some(res.as_ref())
        } else {
            None
        }
    }

    fn is_mapped(&self, index: usize, count: usize) -> bool {
        for i in index..(count + index) {
            if let None = self.subdirs[i] {
                if unsafe { self.as_data_ref() }.get_value(i).is_valid() {
                    return true;
                }
            }
        }
        false
    }

    fn get_mut_or_none(&mut self, index: usize) -> Option<&mut Self> {
        let r = &mut self.subdirs[index];
        if let Some(res) = r {
            Some(res.as_mut())
        } else {
            None
        }
    }
}

// endregion

// region: Boot Page Table

pub static BOOT_PTABLE: RawPageDir = create_boot_ptable();

pub const fn create_boot_ptable() -> RawPageDir {
    let mut root = [PageTableEntry::create_invalid(); PTABLE_ENTRY_COUNT];
    let mut i = 0;
    let half = PTABLE_ENTRY_COUNT / 2;
    while i < half {
        root[i] = PageTableEntry::create(
            PageNum::from_const(i << 18),
            PageTableFlags::PREDEFINED_BOOT,
        ); // 1GiB Page
        root[half + i] = PageTableEntry::create(
            PageNum::from_const(i << 18),
            PageTableFlags::PREDEFINED_BOOT,
        ); // 1GiB Page
        i += 1;
    }
    RawPageDir::from_const(root)
}

// endregion

// region: Page Table Management
pub unsafe fn set_memspace(memspace: impl Deref<Target = MemSpace>) {
    unsafe {
        satp::set(SATP_MODE, memspace.asid, memspace.page_table.ppn().into());
    }
}
// endregion

// region: Kernel MemSpace

lazy_static! {
    pub static ref KERNEL_MEMSPACE: MemSpace = MemSpace::new(
        KERNEL_ASID,
        create_kernel_ptable().expect("Unable to create kernel page table")
    );
}

fn create_kernel_ptable() -> Result<PageTable, PagingError> {
    // kernel sections
    debug_ex!("Creating Kernel Page Table...");
    let sections = [
        (
            Range {
                start: PageNum::from_addr(phys_addr_from_symbol!(_stext)),
                end: PageNum::from_addr(phys_addr_from_symbol!(_etext)),
            },
            PageTableFlags::RX,
        ),
        (
            Range {
                start: PageNum::from_addr(phys_addr_from_symbol!(_srodata)),
                end: PageNum::from_addr(phys_addr_from_symbol!(_erodata)),
            },
            PageTableFlags::R,
        ),
        (
            Range {
                start: PageNum::from_addr(phys_addr_from_symbol!(_sdata)),
                end: PageNum::from_addr(phys_addr_from_symbol!(_edata)),
            },
            PageTableFlags::RW,
        ),
        (
            Range {
                start: PageNum::from_addr(phys_addr_from_symbol!(_sbss)),
                end: PageNum::from_addr(phys_addr_from_symbol!(_ebss)),
            },
            PageTableFlags::RW,
        ),
    ];
    let mut table = PageTable::new().expect("Unable to create the kernel page table.");
    let baseflags = PageTableFlags::VALID
        | PageTableFlags::DIRTY
        | PageTableFlags::ACCESSED
        | PageTableFlags::GLOBAL;
    table.map(
        PageNum::from_addr(KERNEL_OFFSET),
        PageNum::from_const(0),
        MAX_USPACE_ADDR >> PAGE_WIDTH,
        baseflags | PageTableFlags::RWX,
    )?;
    for (range, perm) in sections {
        table.clear(range.start.physical_to_kernel(), range.end - range.start)?;
        table.map(
            range.start.physical_to_kernel(),
            range.start,
            range.end - range.start,
            baseflags | perm,
        )?;
    }
    debug_ex!("Kernel Page Table Created.");
    Ok(table)
}

// endregion

pub fn init() {
    debug_ex!("Initializing paging module in RISC-V.");
    unsafe {
        set_memspace(&KERNEL_MEMSPACE as &MemSpace);
    }
    debug_ex!("Paging module in RISC-V initialized.");
}
