use core::ops::{Deref, DerefMut};

use spin::{MutexGuard, Spin, mutex::Mutex};

use crate::task::preempt::{disable_preempt, restore_preempt};

pub struct SpinLock<T: ?Sized> {
    inner: Mutex<T, Spin>,
}

impl<T: ?Sized> SpinLock<T> {
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.inner.lock()
    }
    pub fn lock_no_preempt(&self) -> NoPreemptSpinLockGuard<'_, T> {
        NoPreemptSpinLockGuard::new(&self.inner)
    }
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> SpinLock<T> {
        SpinLock {
            inner: Mutex::new(value),
        }
    }
}

// region: NoPreemptGuard

pub struct NoPreemptSpinLockGuard<'a, T: ?Sized> {
    inner: Option<MutexGuard<'a, T>>,
}

impl<T: ?Sized> NoPreemptSpinLockGuard<'_, T> {
    pub fn new(mutex: &Mutex<T>) -> NoPreemptSpinLockGuard<'_, T> {
        disable_preempt();
        NoPreemptSpinLockGuard {
            inner: Some(mutex.lock()),
        }
    }
}
impl<T: ?Sized> Drop for NoPreemptSpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.inner = None; // drop first
        restore_preempt();
    }
}
impl<'a, T> Deref for NoPreemptSpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap().deref()
    }
}

impl<'a, T> DerefMut for NoPreemptSpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap().deref_mut()
    }
}

// endregion
