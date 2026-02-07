use alloc::{boxed::Box, vec, vec::Vec};
use core::{cell::UnsafeCell, fmt::Debug, ops::Range};
use spin::RwLock;

pub struct LockedVecStatic<T: ?Sized> {
    lock: RwLock<()>,
    cell: UnsafeCell<Vec<Box<T>>>,
}

unsafe impl<T: ?Sized> Sync for LockedVecStatic<T> {}

impl<T: ?Sized> LockedVecStatic<T> {
    pub const fn new() -> LockedVecStatic<T> {
        LockedVecStatic {
            lock: RwLock::new(()),
            cell: UnsafeCell::new(vec![]),
        }
    }
    pub fn get<'a>(&'a self, index: usize) -> Option<&'a T> {
        let guard = self.lock.read();
        let vec_ptr = self.cell.get();
        let vec = unsafe { &mut *vec_ptr };
        if index >= vec.len() {
            return None;
        }
        let res = vec[index].as_ref();
        drop(guard);
        Some(res)
    }
    pub fn clone<'a>(&'a self) -> Vec<&'a T> {
        let guard = self.lock.read();
        let vec_ptr = self.cell.get();
        let vec = unsafe { &mut *vec_ptr };
        let mut res = vec![];
        for i in 0..vec.len() {
            res.push(vec[i].as_ref());
        }
        drop(guard);
        res
    }
}

impl<T: ?Sized> LockedVecStatic<T> {
    pub fn push_boxed<'a>(&'a self, value: Box<T>) -> (&'a T, usize) {
        let guard = self.lock.write();
        let vec_ptr = self.cell.get();
        let vec = unsafe { &mut *vec_ptr };
        let index = vec.len();
        vec.push(value);
        let res = vec[index].as_ref();
        drop(guard);
        (res, index)
    }
    pub fn append_boxed(&self, values: impl Iterator<Item = Box<T>>) -> Range<usize> {
        let guard = self.lock.write();
        let vec_ptr = self.cell.get();
        let vec = unsafe { &mut *vec_ptr };
        let st = vec.len();
        for value in values {
            vec.push(value);
        }
        let end = vec.len();
        drop(guard);
        st..end
    }
}

impl<T: Sized> LockedVecStatic<T> {
    pub fn push<'a>(&'a self, value: T) -> (&'a T, usize) {
        let guard = self.lock.write();
        let vec_ptr = self.cell.get();
        let vec = unsafe { &mut *vec_ptr };
        let index = vec.len();
        vec.push(Box::new(value));
        let res = vec[index].as_ref();
        drop(guard);
        (res, index)
    }
    pub fn append(&self, values: impl Iterator<Item = T>) -> Range<usize> {
        let guard = self.lock.write();
        let vec_ptr = self.cell.get();
        let vec = unsafe { &mut *vec_ptr };
        let st = vec.len();
        for value in values {
            vec.push(Box::new(value));
        }
        let end = vec.len();
        drop(guard);
        st..end
    }
}

impl<T: Debug> Debug for LockedVecStatic<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.clone()))
    }
}
