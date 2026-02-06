use core::fmt::Debug;
use spin::Once;
use utils::{impl_conversion, impl_deref, range_set::SortedRangeSet};

pub struct MemorySet {
    inner: SortedRangeSet,
}
impl_deref!(MemorySet, SortedRangeSet);
impl_conversion!(MemorySet, SortedRangeSet);

impl MemorySet {
    pub const fn new() -> MemorySet {
        MemorySet {
            inner: SortedRangeSet::new(),
        }
    }
}
impl Debug for MemorySet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for range in self.inner.iter() {
            f.write_fmt(format_args!("[{:#x},{:#x})", range.start, range.end))?;
        }
        Ok(())
    }
}

pub static GENERAL_MEM: Once<MemorySet> = Once::new();

pub fn get_general_memory() -> &'static MemorySet {
    GENERAL_MEM
        .get()
        .unwrap_or_else(|| panic!("Error getting general memory: not initialized."))
}
