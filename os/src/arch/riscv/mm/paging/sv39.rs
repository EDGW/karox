//! RISC-V SV39 Paging
//!
//! SV39 supports up to 39-bit virtual addresses and 44-bit physical addresses.
//! To simplify address translation, the kernel uses a **direct mapping** strategy.
//! The virtual memory layout is designed as follows:
//!
//! |-----------------| 0x0
//! |   User Space    |
//! |-----------------| 0x3f_ffffffff
//! |[ Not Available ]|
//! |-----------------| 0xffffffc0_00000000 [SV39Paging::KERNEL_OFFSET]
//! |  Direct Mapping |
//! |     Space       |
//! |-----------------| [SV39Paging::MMIO_OFFSET]
//! |   MMIO Space    |
//! |-----------------| 0xffffffff_ffffffff
//!
//! [SV39Paging::MMIO_OFFSET] is calculated as [SV39Paging::KERNEL_OFFSET] + [SV39Paging::MAX_PHYSICAL_ADDR].
//! Therefore, the maximum recognizable physical memory size is limited to [SV39Paging::MAX_PHYSICAL_ADDR].
//! Any physical memory exceeding this limit will be ignored.
//!
//! SV39 uses a 3-level page table hierarchy and supports super pages,
//! including 1GiB and 2MiB huge pages.
//! In the direct mapping space, huge pages should be used whenever possible
//! to improve TLB efficiency and overall performance.
//!
//! Paging is initialized in [init_paging()], which sets up the kernel page table.
//! The direct mapping space covers:
//! - IO spaces (usually below 0x80000000 or above the physical memory top),
//! - SBI firmware space (typically between 0x80000000 and 0x80200000),
//! - the kernel itself (loaded at 0x80200000).
//!
//! - IO space and SBI firmware space are treated as general memory regions
//!   with RW permissions. Their safety is enforced by PMP configuration,
//!   not by paging permissions.
//! - The memory occupied by the kernel itself (**referred to as `kbin_space` below**)
//!   is protected with fine-grained page permissions.
//!
//! [init_paging()] calls [setup_kernel_ptable()] to construct the kernel page table.
//!
//! [KERNEL_ROOT_TABLE], [MMIO_TABLES], and [KBIN_SPACE_TABLES] are used to
//! preserve the lifetime of all page tables used by the kernel.

use crate::{
    arch::{
        mm::paging::{PageNum, PageTableEntry, PageTableFlags, PageTableValue},
        symbols::{_ebss, _edata, _erodata, _etext, _sbss, _sdata, _srodata, _stext},
    },
    devices::device_info::MemoryAreaInfo,
    mm::{
        PagingMode,
        frame::{FRAME_ALLOC, FrameAllocatorError, ManagedFrame},
    },
    phys_addr_from_kernel,
};
use alloc::boxed::Box;
use core::{array, usize};
use riscv::register::satp;
use spin::once::Once;

const GB: usize = 1024 * 1024 * 1024;
const MB: usize = 1024 * 1024;
const KB: usize = 1024;

/// SV39 Paging strategy set, implementing [PagingMode].
///
/// This struct defines constants and methods for the SV39 paging strategy, including page sizes,
/// address widths, and memory layout.
pub struct SV39Paging;

/// SV39 memory layout related constants.
#[allow(unused)]
impl SV39Paging {
    /// Level-0 page size: 1GiB.
    pub const PG_SIZE_L0: usize = 1 * GB;
    /// Address width covered by a level-0 page.
    pub const PG_WIDTH_L0: usize = 30;

    /// Level-1 page size: 2MiB.
    const PG_SIZE_L1: usize = 2 * MB;
    /// Address width covered by a level-1 page.
    const PG_WIDTH_L1: usize = 21;

    /// Level-2 page size: 4KiB.
    const PG_SIZE_L2: usize = 4 * KB;
    /// Address width covered by a level-2 page.
    const PG_WIDTH_L2: usize = 12;

    /// Number of level-0 pages reserved for the MMIO space.
    pub const MMIO_PAGE_COUNT: usize = 32;

    /// Number of entries in a page table.
    pub const PTABLE_ENTRY_COUNT: usize = 512;
    const AVAILABLE_ADDR_MASK: usize = (1 << 39) - 1;
}

/// Root page table of the kernel.
///
/// The upper 256 entries are marked as global and shared by all page tables.
pub static KERNEL_ROOT_TABLE: Once<PageTable> = Once::new();

impl PagingMode for SV39Paging {
    const KERNEL_OFFSET: usize = 0xffff_ffc0_0000_0000;
    const MAX_PHYSICAL_ADDR: usize = (256 - Self::MMIO_PAGE_COUNT) * GB;
    const MMIO_OFFSET: usize = Self::KERNEL_OFFSET + Self::MAX_PHYSICAL_ADDR;
    const PAGE_SIZE: usize = Self::PG_SIZE_L2;
    const PAGE_WIDTH: usize = Self::PG_WIDTH_L2;

    /// Initializes paging by setting up the kernel page table.
    fn init() {
        init_paging();
    }
}

pub struct PageTableTracker {
    table_frame: ManagedFrame,
    table_entries: Option<Box<[Option<PageTableTracker>; 512]>>,
    mapped_entries: usize,
}
pub struct PageTable {
    table: PageTableTracker,
}

impl PageTableTracker {
    #[inline(always)]
    fn get_table_ref(&self) -> &mut PageTableValue {
        unsafe { self.table_frame.get_ref::<PageTableValue>() }
    }
    fn get_entries_arr(&mut self) -> &mut [Option<PageTableTracker>; 512] {
        self.table_entries
            .get_or_insert_with(|| Box::new(array::from_fn(|_| None)));
        self.table_entries.as_mut().unwrap()
    }

    unsafe fn fill_or_create(
        &mut self,
        st_index: usize,
        st_vpn_offset: usize,
        ppn: usize,
        count: usize,
        w_offset: usize,
        flags: PageTableFlags,
        dir_flags: PageTableFlags,
    ) -> Result<(), FrameAllocatorError> {
        let prev_entry = self.get_table_ref()[st_index];
        let has_entry = prev_entry != PageTableEntry::create_invalid();
        let prev_ppn = prev_entry.get_ppn().get_value();
        let prev_flags = prev_entry.get_flags();
        let arr = self.get_entries_arr();
        let sub_vpn_offset = st_vpn_offset & ((1 << w_offset) - 1);
        if let Some(tracker) = &mut arr[st_index] {
            // fill
            unsafe {
                return tracker.map_internal(
                    sub_vpn_offset,
                    count,
                    ppn,
                    w_offset - 9,
                    flags,
                    dir_flags,
                );
            }
        } else {
            if has_entry && prev_ppn + sub_vpn_offset == ppn && prev_flags == flags {
                // no need to expand
                return Ok(());
            }
            let frame = FRAME_ALLOC.alloc_managed()?;
            let frame_ppn = frame.get_ppn();
            let mut frame_tracker = PageTableTracker {
                table_frame: frame,
                table_entries: None,
                mapped_entries: 0,
            };
            unsafe {
                if has_entry {
                    // expand
                    frame_tracker.map_internal(
                        0,
                        sub_vpn_offset,
                        prev_ppn,
                        w_offset - 9,
                        prev_flags,
                        dir_flags,
                    )?;
                    frame_tracker.map_internal(
                        sub_vpn_offset,
                        count,
                        ppn,
                        w_offset - 9,
                        flags,
                        dir_flags,
                    )?;
                    frame_tracker.map_internal(
                        sub_vpn_offset + count,
                        (1 << w_offset) - sub_vpn_offset - count,
                        prev_ppn + sub_vpn_offset + count,
                        w_offset - 9,
                        prev_flags,
                        dir_flags,
                    )?;
                } else {
                    // create new
                    frame_tracker.map_internal(
                        sub_vpn_offset,
                        count,
                        ppn,
                        w_offset - 9,
                        flags,
                        dir_flags,
                    )?;
                }
                // automatically destroy allocated frame if failed
            }
            arr[st_index] = Some(frame_tracker);
            self.get_table_ref()[st_index] = PageTableEntry::create(frame_ppn, dir_flags);
            self.mapped_entries += 1;
        }
        Ok(())
    }

    /// **This function assumes that the address range has not been mapped**
    unsafe fn map_internal(
        &mut self,
        vpn_offset: usize,
        count: usize,
        ppn: usize,
        w_offset: usize,
        flags: PageTableFlags,
        dir_flags: PageTableFlags,
    ) -> Result<(), FrameAllocatorError> {
        let st_vpn_offset = vpn_offset;
        // 1) Leaf Page
        if w_offset == 0 {
            for i in 0..count {
                self.get_table_ref()[st_vpn_offset + i] =
                    PageTableEntry::create(PageNum::from_value(ppn + i), flags);
            }
            self.mapped_entries += count;
            return Ok(());
        }
        // 2) Directory Page
        let ed_vpn_offset = vpn_offset + count;
        let st_index = st_vpn_offset >> w_offset;
        let ed_index = ed_vpn_offset >> w_offset;
        let ppn_aligned = ppn >> w_offset << w_offset;
        // 2-1) in a single page
        if st_index == ed_index {
            unsafe {
                self.fill_or_create(
                    st_index,
                    st_vpn_offset,
                    ppn,
                    count,
                    w_offset,
                    flags,
                    dir_flags,
                )?;
            }
            Ok(())
        }
        // 2-2) across multiple pages
        else {
            let st_vo_aligned = st_index << w_offset;
            let ed_vo_aligned = ed_index << w_offset;
            if st_vo_aligned != st_vpn_offset {
                // incomplete first
                unsafe {
                    self.fill_or_create(
                        st_index,
                        st_vpn_offset,
                        ppn,
                        (1 << w_offset) - (st_vpn_offset - st_vo_aligned),
                        w_offset,
                        flags,
                        dir_flags,
                    )?;
                }
            } else {
                // complete first
                self.get_table_ref()[st_index] =
                    PageTableEntry::create(PageNum::from_value(ppn), flags);
                self.mapped_entries += 1;
            }
            let dir_subpage_cnt = 1 << w_offset;
            if ed_vo_aligned != ed_vpn_offset {
                // non-null last
                unsafe {
                    if let Err(err) = self.fill_or_create(
                        ed_index,
                        ed_vo_aligned,
                        ppn_aligned + dir_subpage_cnt * (ed_index - st_index),
                        ed_vpn_offset - ed_vo_aligned,
                        w_offset,
                        flags,
                        dir_flags,
                    ) {
                        self.get_table_ref()[st_index] = PageTableEntry::create_invalid();
                        let arr = self.get_entries_arr();
                        arr[st_index] = None; // resume
                        return Err(err);
                    }
                }
            } else {
                // ingore
            }
            for i in 1..(ed_index - st_index) {
                // middle
                self.get_table_ref()[st_index + i] = PageTableEntry::create(
                    PageNum::from_value(ppn_aligned + dir_subpage_cnt * i),
                    flags,
                );
                self.mapped_entries += 1;
            }
            Ok(())
        }
    }
    fn clean_if_exist(
        &mut self,
        st_index: usize,
        st_vpn_offset: usize,
        count: usize,
        w_offset: usize,
    ) {
        let arr = self.get_entries_arr();
        let sub_vpn_offset = st_vpn_offset & ((1 << w_offset) - 1);
        let mut dispose = false;
        if let Some(tracker) = &mut arr[st_index] {
            tracker.clean_internal(sub_vpn_offset, count, w_offset - 9);
            if tracker.mapped_entries == 0 {
                dispose = true;
            }
        }
        if dispose {
            arr[st_index] = None;
            self.get_table_ref()[st_index] = PageTableEntry::create_invalid();
            self.mapped_entries -= 1;
        }
    }
    fn clean_internal(&mut self, vpn_offset: usize, count: usize, w_offset: usize) {
        let st_vpn_offset = vpn_offset;
        // 1) Leaf Page
        if w_offset == 0 {
            for i in 0..count {
                let table = self.get_table_ref();
                let invalid = PageTableEntry::create_invalid();
                if table[st_vpn_offset + i] != invalid {
                    table[st_vpn_offset + i] = invalid;
                    self.mapped_entries -= 1;
                }
            }
            return;
        }
        // 2) Directory Page
        let ed_vpn_offset = vpn_offset + count;
        let st_index = st_vpn_offset >> w_offset;
        let ed_index = ed_vpn_offset >> w_offset;
        // 2-1) in a single page
        if st_index == ed_index {
            self.clean_if_exist(st_index, st_vpn_offset, count, w_offset);
        }
        // 2-2) across multiple pages
        else {
            let st_vo_aligned = st_index << w_offset;
            let ed_vo_aligned = ed_index << w_offset;
            if st_vo_aligned != st_vpn_offset {
                // incomplete first
                self.clean_if_exist(
                    st_index,
                    st_vpn_offset,
                    (1 << w_offset) - (st_vpn_offset - st_vo_aligned),
                    w_offset,
                );
            } else {
                // complete first
                let table = self.get_table_ref();
                let invalid = PageTableEntry::create_invalid();
                if table[st_index] != invalid {
                    table[st_index] = invalid;
                    self.get_entries_arr()[st_index] = None;
                    self.mapped_entries -= 1;
                }
            }
            if ed_vo_aligned != ed_vpn_offset {
                // non-null last
                self.clean_if_exist(
                    ed_index,
                    ed_vo_aligned,
                    ed_vpn_offset - ed_vo_aligned,
                    w_offset,
                );
            } else {
                // ingore
            }
            for i in 1..(ed_index - st_index) {
                // middle
                let table = self.get_table_ref();
                let invalid = PageTableEntry::create_invalid();
                if table[st_index + i] != invalid {
                    table[st_index + i] = invalid;
                    self.get_entries_arr()[st_index + i] = None;
                    self.mapped_entries -= 1;
                }
            }
        }
    }
}

impl PageTable {
    pub fn new() -> Result<PageTable, FrameAllocatorError> {
        let frame = FRAME_ALLOC.alloc_managed()?;
        Ok(PageTable {
            table: PageTableTracker {
                table_frame: frame,
                table_entries: None,
                mapped_entries: 0,
            },
        })
    }
    pub fn map(
        &mut self,
        vpn: PageNum,
        count: usize,
        ppn: PageNum,
        rwx_flags: PageTableFlags,
        global: bool,
    ) -> Result<(), FrameAllocatorError> {
        unsafe {
            let flags = if global {
                rwx_flags
                    | PageTableFlags::VALID
                    | PageTableFlags::GLOBAL
                    | PageTableFlags::DIRTY
                    | PageTableFlags::ACCESSED
            } else {
                rwx_flags | PageTableFlags::VALID | PageTableFlags::DIRTY | PageTableFlags::ACCESSED
            };
            let dir_flags = if global {
                PageTableFlags::VALID | PageTableFlags::GLOBAL | PageTableFlags::DIR
            } else {
                PageTableFlags::VALID | PageTableFlags::DIR
            };
            self.table.clean_internal(
                vpn.get_value() & (SV39Paging::AVAILABLE_ADDR_MASK >> SV39Paging::PAGE_WIDTH),
                count,
                SV39Paging::PG_WIDTH_L0 - SV39Paging::PG_WIDTH_L2,
            );
            self.table.map_internal(
                vpn.get_value() & (SV39Paging::AVAILABLE_ADDR_MASK >> SV39Paging::PAGE_WIDTH),
                count,
                ppn.get_value(),
                SV39Paging::PG_WIDTH_L0 - SV39Paging::PG_WIDTH_L2,
                flags,
                dir_flags,
            )
        }
    }
    pub unsafe fn map_unchecked(
        &mut self,
        vpn: PageNum,
        count: usize,
        ppn: PageNum,
        rwx_flags: PageTableFlags,
        global: bool,
    ) -> Result<(), FrameAllocatorError> {
        unsafe {
            let flags = if global {
                rwx_flags
                    | PageTableFlags::VALID
                    | PageTableFlags::GLOBAL
                    | PageTableFlags::DIRTY
                    | PageTableFlags::ACCESSED
            } else {
                rwx_flags | PageTableFlags::VALID | PageTableFlags::DIRTY | PageTableFlags::ACCESSED
            };
            let dir_flags = if global {
                PageTableFlags::VALID | PageTableFlags::GLOBAL | PageTableFlags::DIR
            } else {
                PageTableFlags::VALID | PageTableFlags::DIR
            };
            self.table.map_internal(
                vpn.get_value() & (SV39Paging::AVAILABLE_ADDR_MASK >> SV39Paging::PAGE_WIDTH),
                count,
                ppn.get_value(),
                SV39Paging::PG_WIDTH_L0 - SV39Paging::PG_WIDTH_L2,
                flags,
                dir_flags,
            )
        }
    }
}

/// Initializes paging.
pub fn init_paging() {
    setup_kernel_ptable();
}

/// Sets up the kernel page table.
fn setup_kernel_ptable() {
    let mut table = PageTable::new().unwrap_or_else(|err| {
        panic!("Unable to set up the kernel page table: {:?}", err);
    });

    // kernel sections
    let sections = [
        (
            MemoryAreaInfo::from_points(
                phys_addr_from_kernel!(_stext),
                phys_addr_from_kernel!(_etext),
            ),
            PageTableFlags::RX,
        ),
        (
            MemoryAreaInfo::from_points(
                phys_addr_from_kernel!(_srodata),
                phys_addr_from_kernel!(_erodata),
            ),
            PageTableFlags::R,
        ),
        (
            MemoryAreaInfo::from_points(
                phys_addr_from_kernel!(_sdata),
                phys_addr_from_kernel!(_edata),
            ),
            PageTableFlags::RW,
        ),
        (
            MemoryAreaInfo::from_points(
                phys_addr_from_kernel!(_sbss),
                phys_addr_from_kernel!(_ebss),
            ),
            PageTableFlags::RW,
        ),
    ];
    let flat_st = PageNum::from_addr(SV39Paging::KERNEL_OFFSET);
    let flat_ed = PageNum::from_addr(SV39Paging::MMIO_OFFSET);
    table
        .map(
            flat_st,
            flat_ed - flat_st,
            PageNum::from_value(0),
            PageTableFlags::RW,
            true,
        )
        .unwrap_or_else(|err| {
            panic!("Unable to set up the kernel page table: {:?}", err);
        });
    for (section, rwx_flags) in sections {
        let st = PageNum::from_addr(section.start);
        let ed = PageNum::from_addr(section.start + section.length);
        table
            .map(st.physical_to_kernel(), ed - st, st, rwx_flags, true)
            .unwrap_or_else(|err| {
                panic!("Unable to set up the kernel page table: {:?}", err);
            });
    }
    unsafe {
        satp::set(
            satp::Mode::Sv39,
            0,
            table.table.table_frame.get_ppn().get_value(),
        );
    }
    KERNEL_ROOT_TABLE.call_once(|| table);
}
