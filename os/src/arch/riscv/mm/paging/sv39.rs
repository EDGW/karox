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
        mm::{
            config::Paging,
            paging::{PageNum, PageTableEntry, PageTableFlags, PageTableValue, fill_linear_ptable},
        },
        reg::{CrSatp, CrSatpModes, CrSatpValue},
        symbols::{_ebss, _edata, _erodata, _etext, _sbss, _sdata, _skernel, _srodata, _stext},
    },
    devices::device_info::MemoryAreaInfo,
    kserial_println,
    mm::{
        PagingMode,
        frame::{FRAME_ALLOC, ManagedFrames},
    },
    phys_addr_from_kernel,
    utils::{num::AlignableTo, range::Range},
};
use alloc::vec::Vec;
use core::slice::Iter;
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
}

/// Root page table of the kernel.
///
/// The upper 256 entries are marked as global and shared by all page tables.
pub static KERNEL_ROOT_TABLE: Once<ManagedFrames> = Once::new();

/// Level-1 page tables used to map the MMIO space.
///
/// [SV39Paging::MMIO_PAGE_COUNT] tables in total.
pub static MMIO_TABLES: Once<ManagedFrames> = Once::new();

/// Page tables used to map the kernel binary itself (`kbin_space`).
pub static KBIN_SPACE_TABLES: Once<ManagedFrames> = Once::new();

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

/// Initializes paging.
pub fn init_paging() {
    setup_kernel_ptable();
}

/// Converts the frames given in [ManagedFrames] to a page table vector
unsafe fn convert_to_ptable_vec<'a>(frames: &'a ManagedFrames) -> Vec<&'a mut PageTableValue> {
    frames
        .iter()
        .map(|ppn| unsafe {
            &mut *(ppn.physical_to_kernel().get_base_addr() as *mut PageTableValue)
        })
        .collect()
}

/// Sets up the kernel page table.
fn setup_kernel_ptable() {
    // 1. Allocate space for the root page table and MMIO page tables
    let root_table_managed = FRAME_ALLOC.alloc_managed(2).unwrap_or_else(|e| {
        panic!(
            "Failed to allocate space for the kernel root page table: {:?}",
            e
        )
    });
    let root_table = unsafe { root_table_managed.get_ref::<PageTableValue>(0) };
    let mmio_tables_managed = FRAME_ALLOC
        .alloc_managed(SV39Paging::MMIO_PAGE_COUNT)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to allocate space for dynamic mmio page table: {:?}",
                e
            )
        });
    let mut mmio_tables: Vec<&mut PageTableValue> =
        unsafe { convert_to_ptable_vec(&mmio_tables_managed) };

    // 2. Initialize page table entries
    // [0, 255): invalid (user space not mapped here)
    for i in 0..256 {
        root_table[i] = PageTableEntry::create_invalid();
    }

    let entry_flags = PageTableFlags::RWX
        | PageTableFlags::DIRTY
        | PageTableFlags::ACCESSED
        | PageTableFlags::VALID
        | PageTableFlags::GLOBAL;
    let pdir_flags = PageTableFlags::DIR | PageTableFlags::VALID | PageTableFlags::GLOBAL;

    // [256, 512 - MMIO_PAGE_COUNT): linear direct mapping using 1GiB pages
    for i in 0..(256 - SV39Paging::MMIO_PAGE_COUNT) {
        root_table[256 + i] =
            PageTableEntry::create(PageNum::from_addr(i * SV39Paging::PG_SIZE_L0), entry_flags);
    }

    // Kernel binary mapping (kbin_space)
    let kernel_page_index = phys_addr_from_kernel!(_skernel) >> SV39Paging::PG_WIDTH_L0;
    let kernel_tables = create_kbin_space_subtable();
    root_table[256 + kernel_page_index] =
        PageTableEntry::create(kernel_tables.get_ppn(0), pdir_flags);

    // MMIO mapping
    for i in 0..SV39Paging::MMIO_PAGE_COUNT {
        root_table[(512 - SV39Paging::MMIO_PAGE_COUNT) + i] = PageTableEntry::create(
            PageNum::from_addr((&mmio_tables[i]).as_ptr() as usize),
            pdir_flags,
        );
        mmio_tables[i].fill(PageTableEntry::create_invalid());
    }

    // Write SATP register to enable SV39 paging
    CrSatp::set_value(CrSatpValue::create(
        CrSatpModes::SV39,
        0,
        root_table_managed.get_ppn(0),
    ));

    kserial_println!(
        "Kernel page table initialized at {:#x}(ppn {:#x})",
        unsafe { root_table_managed.get_kernel_ptr(0) } as *const u8 as usize,
        root_table_managed.get_ppn(0).get_value()
    );

    // Preserve page table lifetime
    KERNEL_ROOT_TABLE.call_once(|| root_table_managed);
    MMIO_TABLES.call_once(|| mmio_tables_managed);
    KBIN_SPACE_TABLES.call_once(|| kernel_tables);
}

/// Calculate the number of frames required to map the kernel binary itself
fn calc_kbin_space_frames(sections: Iter<Range<usize>>, kstart_aligned: usize) -> usize {
    let mut frame_cnt = 1; // the subtable itself
    let mut frame_allocated = [false; SV39Paging::PTABLE_ENTRY_COUNT];
    for secinfo in sections {
        let sec = secinfo;
        if sec.length == 0 {
            continue;
        }
        let start = sec.start;
        let end = sec.start + sec.length;
        let s_aligned = start.align_down(SV39Paging::PG_SIZE_L1);
        let e_aligned = end.align_down(SV39Paging::PG_SIZE_L1);
        let s_entry_id = (start - kstart_aligned) >> SV39Paging::PG_WIDTH_L1;
        let e_entry_id = (end - kstart_aligned) >> SV39Paging::PG_WIDTH_L1;
        if s_aligned == e_aligned {
            // less than a 2MiB page
            if !frame_allocated[s_entry_id] {
                frame_allocated[s_entry_id] = true;
                frame_cnt += 1;
            }
        } else
        // crossing pages
        {
            if start != s_aligned && !frame_allocated[s_entry_id] {
                // incomplete first page
                frame_allocated[s_entry_id] = true;
                frame_cnt += 1;
            }
            if end != e_aligned && !frame_allocated[e_entry_id] {
                // incomplete last page
                frame_allocated[e_entry_id] = true;
                frame_cnt += 1;
            }
        }
    }
    frame_cnt
}

/// Create the page table subtree for the kernel binary (`kbin_space`)
///
/// The kernel binary is guaranteed to occupy no more than 1GiB of physical memory.
/// Therefore, a single level-1 page table is sufficient to cover this region.
/// Entries corresponding to unused regions are filled with large (2MiB) pages.
fn create_kbin_space_subtable() -> ManagedFrames {
    // All sections and permissions of the kernel
    let sec_perms = [
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

    let kpage_start = phys_addr_from_kernel!(_skernel).align_down(SV39Paging::PG_SIZE_L0);
    let kpage_ppn = PageNum::from_addr(kpage_start);

    // calculate the number of frames needed
    let frame_cnt = calc_kbin_space_frames(sec_perms.map(|x| x.0).iter(), kpage_start);

    // record the frame index of the page table, 0 if it's a large page and doesn't need subtable
    let mut frame_indexies = [0 as u16; SV39Paging::PTABLE_ENTRY_COUNT];

    // allocate frames
    let frames_managed = FRAME_ALLOC
        .alloc_managed(frame_cnt)
        .unwrap_or_else(|e| panic!("Failed to allocate space for kernel page table: {:?}", e));
    let mut frames = unsafe { convert_to_ptable_vec(&frames_managed) };

    // base flags for non-directory entries
    let baseflags = PageTableFlags::DIRTY
        | PageTableFlags::ACCESSED
        | PageTableFlags::VALID
        | PageTableFlags::GLOBAL;
    // flags for directory entries
    let dirflags = PageTableFlags::DIR | PageTableFlags::VALID | PageTableFlags::GLOBAL;

    // flags for entries to the pages not included in the kernel binary.
    let normal_flags = baseflags | PageTableFlags::RWX;

    // fill the table with 2MiB huge entries.
    fill_linear_ptable(
        &mut frames[0],
        0,
        kpage_ppn,
        Paging::PTABLE_ENTRY_COUNT,
        SV39Paging::PG_SIZE_L1,
        normal_flags,
    );

    /// Fill or create level-2 page table entries
    ///
    /// * `frames_managed` and `frame_idx_counter` are used to allocate new frames.
    ///   `frame_idx_counter` always points to the first unallocated frame.
    /// * `frame_idx_recorder` records the allocated frame index for each level-1 entry
    ///   (0 indicates no subtable is allocated).
    /// * `kpage_start` is the base address of the level-0 page containing the kernel.
    /// * `page_addr_start` is the base address of the corresponding level-1 page.
    /// * `addr_start` and `addr_end` specify the exact address range to be mapped.
    fn fill_sec(
        frames_managed: &ManagedFrames,
        frame_idx_recorder: &mut [u16; 512],
        frame_idx_counter: &mut usize,
        kpage_start: usize,
        addr_start: usize,
        page_addr_start: usize,
        addr_end: usize,
        item_flags: PageTableFlags,
        dir_flags: PageTableFlags,
        normal_flags: PageTableFlags,
    ) {
        let mut frames = unsafe { convert_to_ptable_vec(&frames_managed) };

        let entry_id = (addr_start - kpage_start) >> SV39Paging::PG_WIDTH_L1;

        let idx1 = (addr_start - page_addr_start) >> SV39Paging::PG_WIDTH_L2;
        let idx2 = (addr_end - page_addr_start) >> SV39Paging::PG_WIDTH_L2;
        let addr1 = addr_start;
        let addr2 = addr_end;
        if frame_idx_recorder[entry_id] != 0 {
            // add to existing page
            fill_linear_ptable(
                &mut frames[frame_idx_recorder[entry_id] as usize],
                idx1,
                PageNum::from_addr(addr1),
                idx2 - idx1,
                SV39Paging::PG_SIZE_L2,
                item_flags,
            );
        } else {
            // create a new page
            frames[0][entry_id] =
                PageTableEntry::create(frames_managed.get_ppn(*frame_idx_counter), dir_flags);
            frame_idx_recorder[entry_id] = *frame_idx_counter as u16;
            if idx1 != 0 {
                fill_linear_ptable(
                    &mut frames[*frame_idx_counter],
                    0,
                    PageNum::from_addr(page_addr_start),
                    idx1,
                    SV39Paging::PG_SIZE_L2,
                    normal_flags,
                );
            }
            fill_linear_ptable(
                &mut frames[*frame_idx_counter],
                idx1,
                PageNum::from_addr(addr1),
                idx2 - idx1,
                SV39Paging::PG_SIZE_L2,
                item_flags,
            );
            if idx2 != SV39Paging::PTABLE_ENTRY_COUNT {
                fill_linear_ptable(
                    &mut frames[*frame_idx_counter],
                    idx2,
                    PageNum::from_addr(addr2),
                    SV39Paging::PTABLE_ENTRY_COUNT - idx2,
                    SV39Paging::PG_SIZE_L2,
                    normal_flags,
                );
            }
            *frame_idx_counter += 1;
        }
    }

    let mut idx = 1;

    for secinfo in sec_perms {
        let sec = secinfo.0;
        if sec.length == 0 {
            continue;
        }
        let perm = secinfo.1;
        let start = sec.start;
        let end = sec.start + sec.length;
        let s_aligned = start.align_down(SV39Paging::PG_SIZE_L1);
        let e_aligned = end.align_down(SV39Paging::PG_SIZE_L1);
        if s_aligned == e_aligned {
            // single page
            fill_sec(
                &frames_managed,
                &mut frame_indexies,
                &mut idx,
                kpage_start,
                start,
                s_aligned,
                end,
                baseflags | perm,
                dirflags,
                normal_flags,
            );
        } else {
            // cross pages
            if start != s_aligned {
                // first page
                fill_sec(
                    &frames_managed,
                    &mut frame_indexies,
                    &mut idx,
                    kpage_start,
                    start,
                    s_aligned,
                    s_aligned + SV39Paging::PG_SIZE_L1,
                    baseflags | perm,
                    dirflags,
                    normal_flags,
                );
            }
            if end != e_aligned {
                // last page
                fill_sec(
                    &frames_managed,
                    &mut frame_indexies,
                    &mut idx,
                    kpage_start,
                    e_aligned,
                    e_aligned,
                    end,
                    baseflags | perm,
                    dirflags,
                    normal_flags,
                );
            }
            let fill_st = ((start - kpage_start) >> SV39Paging::PG_WIDTH_L1) + 1;
            let fill_ed = (end - kpage_start) >> SV39Paging::PG_WIDTH_L1;
            for i in fill_st..fill_ed {
                // middle pages
                frames[0][i] = PageTableEntry::create(
                    PageNum::from_addr(kpage_start + i * SV39Paging::PG_SIZE_L1),
                    baseflags | perm,
                );
            }
        }
    }
    frames_managed
}
