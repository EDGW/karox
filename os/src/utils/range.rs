//! This module provides a `Range<T>` struct and extensions for range operations.
//!
//! It defines the `RangeExt` trait which adds utility methods to the standard `Range<T>` type,
//! including overlap checking and range subtraction operations.

use core::{
    ops::{Add, Range, Sub},
};

pub trait RangeExt<T> :  
    where Self: Sized,T: Copy + Add<Output = T> + Sub<Output = T> + PartialOrd
    {
    /// Check if this range overlaps with another range.
    /// 
    /// Always return false if either range is empty.
    fn overlap(&self, another: &Self) -> bool;
    /// Subtracts another range from this range, returning the remaining segments.
    /// The output is a fixed-size array `[Option<Range<T>>; 2]` where:
    /// - `[Some(left), Some(right)]`: Self fully contains rhs (split into two non-overlapping segments)
    /// - `[Some(remaining), None]`: Partial overlap (only one valid segment remains) or no overlap (returns original self)
    /// - `[None, None]`: Self is fully contained within rhs (no remaining range) or self is empty
    /// 
    /// **This function is safe only when `T` is an unsigned value.**
    fn sub(&self, rhs: &Self) -> [Option<Range<T>>; 2];
}

/// Implementation of `RangeExt` for the standard library's `Range<T>`.
impl<T> RangeExt<T> for Range<T> 
where T: Copy + Add<Output = T> + Sub<Output = T> + PartialEq + Ord {

    #[inline(always)]
    fn overlap(&self, another: &Range<T>)->bool
    {
        if self.is_empty() || another.is_empty() {
            return false;
        }
        let self_l = self.start;
        let self_r = self.end;
        let another_l = another.start;
        let another_r = another.end;
        if self_r <= another_l {
            return false;
        }
        if self_l >= another_r {
            return false;
        }
        return true;
    }
    fn sub(&self, rhs: &Self) -> [Option<Range<T>>; 2]{
        let self_left = self.start;
        let self_right = self.end;
        let rhs_left = rhs.start;
        let rhs_right = rhs.end;
        if self.overlap(&rhs) {
            // [     self    )
            //    [  rhs  )
            // [r1)       [r2)
            if self_left < rhs_left && self_right > rhs_right {
                [
                    Some(Range {
                        start: self_left,
                        end: rhs_left,
                    }),
                    Some(Range {
                        start: rhs_right,
                        end: self_right,
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
                        end: rhs_left,
                    }
                }
                //      [    self    )
                // [ -- [    rhs  )--)
                //                [r1) or empty
                else {
                    Range {
                        start: rhs_right,
                        end: self_right,
                    }
                };
                if res.is_empty() {
                    [None, None]
                } else {
                    [Some(res), None]
                }
            }
        } else if !self.is_empty() {
            [Some(self.clone()), None]
        } else {
            [None, None]
        }
    }
}
