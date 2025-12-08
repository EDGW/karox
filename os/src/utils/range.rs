//! This module defines a [Range<T>] struct for convenience

use core::{
    fmt::Debug,
    ops::{Add, Sub},
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// C-style aligned range structure for representing start/length numerical pairs.
///
/// This struct models a **left-closed, right-open interval** `[start, start + length)`, where:
/// - `start` is the inclusive left boundary of the range;
/// - `start + length` is the exclusive right boundary of the range.
///
/// The `#[repr(C)]` attribute ensures C-compatible memory layout for interoperability with C code.
pub struct Range<T: Debug> {
    /// Start address (or index) of the range.
    pub start: T,
    /// Length (size) of the range.
    pub length: T,
}

impl<T: Debug> Range<T> {

    pub fn from_points(start: T, end: T) -> Range<T>
    where T: Sub<Output = T> + Copy
    {
        Range{
            start: start,
            length: end - start
        }
    }

    /// Determines if this range overlaps with another range.
    ///
    /// Overlap rules for left-closed, right-open intervals (`[start, start + length)`):
    /// - If either range is empty (length = 0), returns `false` immediately (no overlap possible).
    /// - For non-empty ranges: overlap is confirmed if the ranges are not completely disjoint.
    ///
    /// # Generic Constraints
    /// `T` must support copy semantics, addition/subtraction (to calculate interval boundaries),
    /// and partial ordering (to compare boundary values).
    #[inline(always)]
    pub fn overlap(&self, another: &Range<T>) -> bool
    where
        T: Copy + Add<Output = T> + Sub<Output = T> + PartialOrd,
    {
        if self.empty() || another.empty() {
            return false;
        }
        let self_l = self.start;
        let self_r = self_l + self.length;
        let another_l = another.start;
        let another_r = another_l + another.length;
        if self_r <= another_l {
            return false;
        }
        if self_l >= another_r {
            return false;
        }
        return true;
    }

    /// Checks if the range is empty.
    pub fn empty(&self) -> bool
    where
        T: Add<Output = T> + PartialEq + Copy,
    {
        self.start + self.length == self.start
    }
}


impl<T: Debug + Copy + Add<Output = T> + Sub<Output = T> + Ord + PartialEq> Sub for Range<T> {
    type Output = [Option<Range<T>>; 2];
    
    /// Subtracts the `rhs` range from `self` and returns up to two non-overlapping remaining segments.
    /// The output is a fixed-size array `[Option<Range<T>>; 2]` where:
    /// - `[Some(left), Some(right)]`: Self fully contains rhs (split into two non-overlapping segments)
    /// - `[Some(remaining), None]`: Partial overlap (only one valid segment remains) or no overlap (returns original self)
    /// - `[None, None]`: Self is fully contained within rhs (no remaining range) or self is empty
    /// ### Notes
    /// - This function is safe onlu when `T` is an unsigned value.
    fn sub(self, rhs: Self) -> Self::Output{
        let self_left = self.start;
        let self_right = self.start + self.length;
        let rhs_left = rhs.start;
        let rhs_right = rhs.start + rhs.length;
        if self.overlap(&rhs) {
            // [     self    )
            //    [  rhs  )
            // [r1)       [r2)
            if self_left < rhs_left && self_right > rhs_right {
                [
                    Some(Range {
                        start: self_left,
                        length: rhs_left - self_left,
                    }),
                    Some(Range {
                        start: rhs_right,
                        length: self_right - rhs_right,
                    }),
                ]
            }
            //    [  self  )
            // [      rhs      )
            else if self_left > rhs_left && self_right < rhs_right {
                [None, None]
            } else {
                let res = 
                // [    self    )
                // [--[ rhs     ) -- )
                // [r1)
                if self_left <= rhs_left {
                    Range {
                        start: self_left,
                        length: rhs_left - self_left,
                    }
                }
                //      [    self    )
                // [ -- [    rhs  )--)
                //                [r1) or empty
                else {
                    Range {
                        start: rhs_right,
                        length: self_right - rhs_right
                    }
                };
                if res.empty() {
                    [None, None]
                } else {
                    [Some(res), None]
                }
            }
        } else if !self.empty() {
            [Some(self), None]
        } else {
            [None, None]
        }
    }
}
