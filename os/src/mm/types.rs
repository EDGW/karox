//! This module defines some useful pointer types

use alloc::{boxed::Box, vec::Vec};
use core::{fmt::Display, ops::Deref};

/// A "maybe" owned object.
///
/// - If the packed object is a [Box] pointer, it will be disposed as the life cycle of this pointer ends.
/// - If the packed object is static, it keeps alive permanently.
#[derive(Debug)]
pub enum MaybeOwned<T: ?Sized + 'static> {
    /// Referencing a static value
    Static(&'static T),
    /// Functions as a boxed pointer
    Boxed(Box<T>),
}

impl<T: ?Sized> Deref for MaybeOwned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwned::Static(s) => s,
            MaybeOwned::Boxed(b) => b,
        }
    }
}

/// Packed type for [MaybeOwned<str>]
pub type MaybeOwnedStr = MaybeOwned<str>;

/// Packed type for [MaybeOwned<[u8]]>]
pub type MaybeOwnedBytes = MaybeOwned<[u8]>;

impl<T: ?Sized + Display> Display for MaybeOwned<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Boxed(bx) => bx.fmt(f),
            Self::Static(st) => st.fmt(f),
        }
    }
}

/// Implement the split function for arrays packed in [MaybeOwned] that are comparable
impl<T: PartialEq> MaybeOwned<[T]> {
    /// Split the vector in place at a specific separator
    pub fn split_by(&self, split: T) -> Vec<&[T]> {
        let r = self.as_ref();
        let mut s = 0;
        let mut e = 0;
        let mut vec: Vec<&[T]> = Vec::<&[T]>::new();
        for c in r {
            if *c == split {
                vec.push(&r[s..e]);
                e += 1;
                s = e;
                continue;
            }
            e += 1;
        }
        if s != e {
            vec.push(&r[s..e]);
        }
        vec
    }
}
