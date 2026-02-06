//! Numeric Utilities
use core::ops::{Add, Rem, Sub};

/// A trait for aligning numerical values.
///
/// Provides methods to align values up or down to the nearest multiple of a given alignment.
pub trait AlignableTo {
    /// Aligns the value up to the nearest multiple of `align`.
    fn align_up(self, align: Self) -> Self;

    /// Aligns the value down to the nearest multiple of `align`.
    fn align_down(self, align: Self) -> Self;
}

impl<T> AlignableTo for T
where
    T: Copy + Rem<Output = T> + Add<Output = T> + PartialEq<T> + Default + Sub<Output = T>,
{
    fn align_up(self, align: Self) -> Self {
        if self % align == T::default() {
            self
        } else {
            self + (align - (self % align))
        }
    }
    fn align_down(self, align: Self) -> Self {
        if self % align == T::default() {
            self
        } else {
            self - (self % align)
        }
    }
}
