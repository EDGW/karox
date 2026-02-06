//! Uniprocessor interior mutability primitives
use core::{cell::{Ref, RefCell, RefMut}, fmt::Debug};

/// Wrap a static data structure inside it so that we are
/// able to access it without any `unsafe`.
///
/// We should only use it in uniprocessor.
///
/// In order to get mutable reference of inner data, call
/// `exclusive_access`.
/// 
/// It's adapted from `UPSafeCell` in rCore project, and renamed.
pub struct LocalCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for LocalCell<T> {}

impl<T> LocalCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    pub const unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Exclusive access inner data in UPSafeCell. Panic if the data has been borrowed.
    pub unsafe fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }

    /// Shared access inner data in UPSafeCell. Panic if the data has been mutably borrowed.
    pub unsafe fn access(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }
}

impl<T:Debug> Debug for LocalCell<T>{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UPSafeCell").field("inner", &self.inner).finish()
    }
}