//! Paging Module
//! It's arch-specific implementations shall implement such members:
//!
//! ## Page Table Design
//! The arch-specific paging system should implement [PageDirTrait] as [PageDir].
//!
//! A [PageTable] contains a root [PageDir] and provides functions to manage the page table.

use utils::num::AlignableTo;

use crate::{
    arch::{
        self, PTABLE_MAX_LEVEL,
        mm::{
            PageNum,
            paging::{PageDir, PageTableFlags},
        },
    }, debug_ex, mm::{
        config::PTABLE_ENTRY_COUNT,
        frame::{FRAME_ALLOC, FrameAllocatorError},
    }
};
use core::fmt::Debug;

// region: PageDirTrait
pub trait PageDirTrait: Sized {
    /// Number of bits used to find a entry in the page dirs of a layer
    const LEVEL_WIDTH: usize;

    /// Number of entries in a page dir.
    const ENTRY_COUNT: usize = 1 << Self::LEVEL_WIDTH;

    /// Should ppn have the same alignment with ppn.
    ///
    /// This constant will affect the behavior of [PageTable::map] by default.
    const PPN_ALIGNED: bool;

    /// Fill a range that is mapped to `ppn` advancing by `step` with `flags`.
    unsafe fn fill(
        &mut self,
        index: usize,
        ppn: PageNum,
        count: usize,
        step: usize,
        flags: PageTableFlags,
    );

    /// Get a the subdir object. If the given index is mapped to a huge page, expand the page to a subdir.
    ///
    /// **This operation should not change the mapping whether on success of failure.**
    fn get_or_expand(
        &mut self,
        index: usize,
        expand_step: usize,
    ) -> Result<&mut Self, FrameAllocatorError>;

    /// Get a the subdir object. If the given index is mapped to a huge page or is a leaf page, return [None].
    fn get_or_none(&self, index: usize) -> Option<&Self>;

    /// Get a the subdir object. If the given index is mapped to a huge page or is a leaf page, return [None].
    fn get_mut_or_none(&mut self, index: usize) -> Option<&mut Self>;

    /// Unfill a range. **If the range contains subdirs, remove the subdir.**
    fn unfill(&mut self, index: usize, count: usize);

    /// Whether at least 1 page in range is mapped.
    /// **If an entry points to a subdir, it is seen as not mapped.**
    fn is_mapped(&self, index: usize, count: usize) -> bool;
}
// endregion

// region: PageTable
pub struct PageTable {
    root: PageDir,
}
impl Debug for PageTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.root))
    }
}

impl PageTable {
    pub fn new() -> Result<PageTable, PagingError> {
        match FRAME_ALLOC.alloc_managed() {
            Ok(frame) => Ok(PageTable {
                root: unsafe { PageDir::new_empty(frame) },
            }),
            Err(error) => Err(PagingError::FrameAllocatorError { error }),
        }
    }
    pub fn map(
        &mut self,
        vpn: PageNum,
        ppn: PageNum,
        count: usize,
        flags: PageTableFlags,
    ) -> Result<(), PagingError> {
        if is_mapped_internal(&mut self.root, vpn.into(), count, PTABLE_MAX_LEVEL) {
            return Err(PagingError::ConflictMappingError);
        }
        match unsafe {
            map_pages_internal(
                &mut self.root,
                vpn.into(),
                ppn.into(),
                count,
                flags,
                PTABLE_MAX_LEVEL,
            )
        } {
            Ok(()) => Ok(()),
            Err(error) => {
                clear_pages_on_failure(&mut self.root, vpn.into(), count, PTABLE_MAX_LEVEL);
                Err(PagingError::FrameAllocatorError { error })
            }
        }
    }
    pub fn clear(&mut self, vpn: PageNum, count: usize) -> Result<(), PagingError> {
        if let Err(error) = expand_pages(&mut self.root, vpn.into_const(), count, PTABLE_MAX_LEVEL)
        {
            return Err(PagingError::FrameAllocatorError { error });
        }
        unsafe {
            clear_pages_internal(&mut self.root, vpn.into_const(), count, PTABLE_MAX_LEVEL);
        }
        Ok(())
    }
    pub fn ppn(&self) -> PageNum {
        self.root.ppn()
    }
}

fn calc_index(vpn: usize, level_offset: usize, level_width: usize, non_zero: bool) -> usize {
    let res = (vpn >> level_offset) & ((1 << level_width) - 1);
    if non_zero && res == 0 {
        return PTABLE_ENTRY_COUNT;
    }
    res
}

fn check_subpage(
    table: &PageDir,
    index: usize,
    vpn_sub: usize,
    count_sub: usize,
    level_sub: usize,
) -> bool {
    if table.is_mapped(index, 1) {
        return true;
    };
    if let Some(sub_page) = table.get_or_none(index) {
        return is_mapped_internal(sub_page, vpn_sub, count_sub, level_sub);
    }
    return false;
}

fn is_mapped_internal(
    table: &PageDir,
    // Common parameters
    vpn: usize,
    count: usize,
    // Table level parameters
    level: usize,
) -> bool {
    if count == 0 {
        return false;
    }
    let level_width = PageDir::LEVEL_WIDTH;
    let level_offset = level * level_width;
    let subpg_size = 1 << level_offset; // in pn

    // leaf page table
    if level == 0 {
        let index = calc_index(vpn, level_offset, level_width, false);
        if table.is_mapped(index, count) {
            return true;
        }
        return false;
    }
    // directory table
    //  [    fill range     )
    // ... | .. | .. | .. | ..
    //  [ F)              [E)
    //     [      M       )
    let ad_st = vpn.align_down(subpg_size);
    let au_st = vpn.align_up(subpg_size);
    let ed = vpn + count;
    let ad_ed = ed.align_down(subpg_size);
    // Case 1: Within a page and less than a page
    // .. | .. | ..
    //    [[[ )
    if ad_st == ad_ed {
        let index = calc_index(vpn, level_offset, level_width, false);
        return check_subpage(table, index, vpn, count, level - 1);
    }
    // Case 2: Crossing pages or exactly taking a full page

    //    st          end
    //  |  *  | ... |  *  |
    //  ad    au    ad    au
    if vpn != ad_st {
        // first page
        let index = calc_index(vpn, level_offset, level_width, false);
        if check_subpage(table, index, vpn, au_st - vpn, level - 1) {
            return true;
        }
    }
    if ed != ad_ed {
        // last page
        let index = calc_index(ed, level_offset, level_width, false);
        if check_subpage(table, index, ad_ed, ed - ad_ed, level - 1) {
            return true;
        }
    }
    // expand
    let index_st = calc_index(au_st, level_offset, level_width, false);
    let index_ed = calc_index(ad_ed, level_offset, level_width, true);
    if table.is_mapped(index_st, index_ed - index_st) {
        return true;
    }
    for i in 0..(index_ed - index_st) {
        if let Some(sub_page) = table.get_or_none(i + index_st) {
            if is_mapped_internal(sub_page, au_st + subpg_size * i, subpg_size, level - 1) {
                return true;
            }
        }
    }
    false
}

fn clear_pages_on_failure(
    table: &mut PageDir,
    // Common parameters
    vpn: usize,
    count: usize,
    // Table level parameters
    level: usize,
) {
    if count == 0 {
        return;
    }
    let level_width = PageDir::LEVEL_WIDTH;
    let level_offset = level * level_width;
    let subpg_size = 1 << level_offset; // in pn

    // leaf page table
    if level == 0 {
        let index = calc_index(vpn, level_offset, level_width, false);
        table.unfill(index, count / subpg_size);
        return;
    }
    // directory table
    //  [    fill range     )
    // ... | .. | .. | .. | ..
    //  [ F)              [E)
    //     [      M       )
    let ad_st = vpn.align_down(subpg_size);
    let au_st = vpn.align_up(subpg_size);
    let ed = vpn + count;
    let ad_ed = ed.align_down(subpg_size);
    // Case 1: Within a page and less than a page
    // .. | .. | ..
    //    [[[ )
    if ad_st == ad_ed {
        if let Some(sub_page) =
            table.get_mut_or_none(calc_index(vpn, level_offset, level_width, false))
        {
            clear_pages_on_failure(sub_page, vpn, count, level - 1)
        }
        return;
    }
    // Case 2: Crossing pages or exactly taking a full page

    //    st          end
    //  |  *  | ... |  *  |
    //  ad    au    ad    au
    if vpn != ad_st {
        // first page
        if let Some(sub_page) =
            table.get_mut_or_none(calc_index(vpn, level_offset, level_width, false))
        {
            clear_pages_on_failure(sub_page, vpn, au_st - vpn, level - 1);
        }
    }
    if ed != ad_ed {
        // last page
        if let Some(sub_page) =
            table.get_mut_or_none(calc_index(ed, level_offset, level_width, false))
        {
            clear_pages_on_failure(sub_page, ad_ed, ed - ad_ed, level - 1);
        }
    }
    // expand
    let index_st = calc_index(au_st, level_offset, level_width, false);
    let index_ed = calc_index(ad_ed, level_offset, level_width, true);
    table.unfill(index_st, index_ed - index_st);
    return;
}

fn expand_pages(
    table: &mut PageDir,
    vpn: usize,
    count: usize,
    level: usize,
) -> Result<(), FrameAllocatorError> {
    if count == 0 {
        return Ok(());
    }
    let level_width = PageDir::LEVEL_WIDTH;
    let level_offset = level * level_width;
    let subpg_size = 1 << level_offset; // in pn
    if level == 0 {
        return Ok(());
    }

    let subsubpage_size = 1 << ((level - 1) * level_width);
    // directory table
    //  [    fill range     )
    // ... | .. | .. | .. | ..
    //  [ F)              [E)
    //     [      M       )
    let ad_st = vpn.align_down(subpg_size);
    let au_st = vpn.align_up(subpg_size);
    let ed = vpn + count;
    let ad_ed = ed.align_down(subpg_size);
    // Case 1: Within a page and less than a page
    // .. | .. | ..
    //    [[[ )
    if ad_st == ad_ed {
        let sub_page = table.get_or_expand(
            calc_index(vpn, level_offset, level_width, false),
            subsubpage_size,
        )?;
        return expand_pages(sub_page, vpn, count, level - 1);
    }
    // Case 2: Crossing pages or exactly taking a full page

    //    st          end
    //  |  *  | ... |  *  |
    //  ad    au    ad    au
    if vpn != ad_st {
        // first page
        let sub_page = table.get_or_expand(
            calc_index(vpn, level_offset, level_width, false),
            subsubpage_size,
        )?;
        expand_pages(sub_page, vpn, au_st - vpn, level - 1)?;
    }
    if ed != ad_ed {
        // last page
        let sub_page = table.get_or_expand(
            calc_index(ed, level_offset, level_width, false),
            subsubpage_size,
        )?;

        expand_pages(sub_page, ad_ed, ed - ad_ed, level - 1)?;
    }
    Ok(())
}

unsafe fn clear_pages_internal(
    table: &mut PageDir,
    // Common parameters
    vpn: usize,
    count: usize,
    // Table level parameters
    level: usize,
) {
    if count == 0 {
        return;
    }
    let level_width = PageDir::LEVEL_WIDTH;
    let level_offset = level * level_width;
    let subpg_size = 1 << level_offset; // in pn

    // leaf page table
    if level == 0 {
        let index = calc_index(vpn, level_offset, level_width, false);
        table.unfill(index, count / subpg_size);
        return;
    }
    let subsubpage_size = 1 << ((level - 1) * level_width);
    // directory table
    //  [    fill range     )
    // ... | .. | .. | .. | ..
    //  [ F)              [E)
    //     [      M       )
    let ad_st = vpn.align_down(subpg_size);
    let au_st = vpn.align_up(subpg_size);
    let ed = vpn + count;
    let ad_ed = ed.align_down(subpg_size);
    // Case 1: Within a page and less than a page
    // .. | .. | ..
    //    [[[ )
    if ad_st == ad_ed {
        let sub_page = table
            .get_or_expand(
                calc_index(vpn, level_offset, level_width, false),
                subsubpage_size,
            )
            .unwrap();
        return unsafe { clear_pages_internal(sub_page, vpn, count, level - 1) };
    }
    // Case 2: Crossing pages or exactly taking a full page

    //    st          end
    //  |  *  | ... |  *  |
    //  ad    au    ad    au
    if vpn != ad_st {
        // first page
        let sub_page = table
            .get_or_expand(
                calc_index(vpn, level_offset, level_width, false),
                subsubpage_size,
            )
            .unwrap();
        unsafe { clear_pages_internal(sub_page, vpn, au_st - vpn, level - 1) };
    }
    if ed != ad_ed {
        // last page
        let sub_page = table
            .get_or_expand(
                calc_index(ed, level_offset, level_width, false),
                subsubpage_size,
            )
            .unwrap();
        unsafe { clear_pages_internal(sub_page, ad_ed, ed - ad_ed, level - 1) };
    }
    let index_st = calc_index(au_st, level_offset, level_width, false);
    let index_ed = calc_index(ad_ed, level_offset, level_width, true);
    table.unfill(index_st, index_ed - index_st);
}

/// Internal method to map pages in a subtable.
/// ### Parameters
/// * map `count` pages at `vpn` to `ppn`,
///     assuming that **the given vpn would not exceed the valid vpn range of this subtable**.
/// * `eflags` and `dflags`: flags for page leaf entry(including huge pages) and page dir entry.
/// * `level`: the level of this page tracker.
///     Root page has the highest level while a level of 0 indicates a leaf page.
///
/// **This method is unsafe because we assume that the given `vpn` value is within in range of this subtable**
///
/// **When failed, the already-mapped entries will be kept. They should be manually cleared.**
unsafe fn map_pages_internal(
    table: &mut PageDir,
    // Common parameters
    vpn: usize,
    ppn: usize,
    count: usize,
    flags: PageTableFlags,
    // Table level parameters
    level: usize,
) -> Result<(), FrameAllocatorError> {
    if count == 0 {
        return Ok(());
    }
    let level_width = PageDir::LEVEL_WIDTH;
    let level_offset = level * level_width;
    let subpg_size = 1 << level_offset; // in pn

    // leaf page table
    if level == 0 {
        let index = calc_index(vpn, level_offset, level_width, false);
        unsafe {
            table.fill(
                index,
                PageNum::from(ppn),
                count / subpg_size,
                subpg_size,
                flags,
            );
        }
        return Ok(());
    }
    let subsubpage_size = 1 << ((level - 1) * level_width);
    // directory table
    //  [    fill range     )
    // ... | .. | .. | .. | ..
    //  [ F)              [E)
    //     [      M       )
    let ad_st = vpn.align_down(subpg_size);
    let au_st = vpn.align_up(subpg_size);
    let ed = vpn + count;
    let ad_ed = ed.align_down(subpg_size);
    // Case 1: Within a page and less than a page
    // .. | .. | ..
    //    [[[ )
    if ad_st == ad_ed {
        let sub_page = table.get_or_expand(
            calc_index(vpn, level_offset, level_width, false),
            subsubpage_size,
        )?;
        return unsafe { map_pages_internal(sub_page, vpn, ppn, count, flags, level - 1) };
    }
    // Case 2: Crossing pages or exactly taking a full page

    //    st          end
    //  |  *  | ... |  *  |
    //  ad    au    ad    au
    if vpn != ad_st {
        // first page
        let sub_page = table.get_or_expand(
            calc_index(vpn, level_offset, level_width, false),
            subsubpage_size,
        )?;
        unsafe { map_pages_internal(sub_page, vpn, ppn, au_st - vpn, flags, level - 1) }?;
    }
    if ed != ad_ed {
        // last page
        let sub_page = table.get_or_expand(
            calc_index(ed, level_offset, level_width, false),
            subsubpage_size,
        )?;
        unsafe {
            map_pages_internal(
                sub_page,
                ad_ed,
                ppn + (ad_ed - vpn),
                ed - ad_ed,
                flags,
                level - 1,
            )
        }?;
    }
    let mid_ppn_st = ppn + (au_st - vpn);
    if !PageDir::PPN_ALIGNED || mid_ppn_st % subpg_size == 0 {
        // align matched
        unsafe {
            table.fill(
                calc_index(au_st, level_offset, level_width, false),
                PageNum::from(mid_ppn_st),
                (ad_ed - au_st) / subpg_size,
                subpg_size,
                flags,
            );
        }
    } else {
        // expand
        let index_st = calc_index(au_st, level_offset, level_width, false);
        let index_ed = calc_index(ad_ed, level_offset, level_width, true);
        for i in 0..(index_ed - index_st) {
            let sub_page = table.get_or_expand(i + index_st, subsubpage_size)?;
            unsafe {
                map_pages_internal(
                    sub_page,
                    au_st + subpg_size * i,
                    ppn + (au_st + subpg_size * i - vpn),
                    subpg_size,
                    flags,
                    level - 1,
                )
            }?;
        }
    }
    Ok(())
}

// endregion

pub fn init() {
    debug_ex!("Initializing paging module...");
    arch::mm::paging::init();
    debug_ex!("Paging module initialized...");
}

// region: Errors

#[derive(Debug)]
pub enum PagingError {
    FrameAllocatorError { error: FrameAllocatorError },
    ConflictMappingError,
}

// endregion
