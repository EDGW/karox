use core::{
    cell::UnsafeCell,
    ptr::{read_volatile, write_volatile},
};

#[repr(transparent)]
pub struct Register<T: Sized + Copy> {
    inner: UnsafeCell<T>,
}

impl<T: Sized + Copy> Register<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { read_volatile(self.inner.get()) }
    }
    #[inline(always)]
    pub fn write(&self, value: T) {
        unsafe {
            write_volatile(self.inner.get(), value);
        }
    }
}
