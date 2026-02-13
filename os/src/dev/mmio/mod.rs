use core::{fmt::Debug, ops::Range};
use utils::impl_basic;
pub mod reg;

pub struct IoRange {
    inner: Range<usize>,
}

impl_basic!(IoRange, Range<usize>);

impl IoRange {
    pub fn validate<T: Sized>(&self, val_type: IoRangeValidationType) -> bool {
        let self_size = self.len();
        let size = size_of::<T>();
        match val_type {
            IoRangeValidationType::Fit => self_size == size,
            IoRangeValidationType::Compatible => self_size >= size,
        }
    }
}

impl Clone for IoRange {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Debug for IoRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("[{:#x},{:#x})", self.start, self.end))
    }
}

pub enum IoRangeValidationType {
    /// The size of the IO range is exactly the same as the size of io memmap struct
    Fit,
    /// The size of the IO range equal or is greater than the size of io memmap struct
    Compatible,
}
