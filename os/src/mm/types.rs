use core::{fmt::Display, ops::Deref};

use alloc::{boxed::Box, vec::Vec};

#[derive(Debug)]
pub enum MaybeOwned<T: ?Sized + 'static> {
    Static(&'static T),
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

pub type MaybeOwnedStr = MaybeOwned<str>;
pub type MaybeOwnedBytes = MaybeOwned<[u8]>;

impl<T: ?Sized + Display> Display for MaybeOwned<T>{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self{
            Self::Boxed(bx) => bx.fmt(f),
            Self::Static(st) => st.fmt(f) 
        }
    }
}

impl<T:PartialEq> MaybeOwned<[T]>{
    pub fn split_by(&self, split: T) -> Vec<&[T]>{
        let r = self.as_ref();
        let mut s = 0;
        let mut e = 0;
        let mut vec: Vec<&[T]> = Vec::<&[T]>::new();
        for c in r{
            if *c == split{
                vec.push(&r[s..e]);
                e += 1;
                s = e;
                continue;
            }
            e += 1;
        }
        if s != e{
            vec.push(&r[s..e]);
        }
        vec
    } 
}