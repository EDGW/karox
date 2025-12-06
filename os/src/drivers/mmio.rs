use core::ptr::{read_volatile, write_volatile};

pub struct Register<T: Sized + Copy>(T);

impl<T: Sized + Copy> Register<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { read_volatile(self as *const Self as *const T) }
    }
    #[inline(always)]
    pub fn write(&mut self, value: T) {
        unsafe {
            write_volatile(self as *const Register<T> as *mut T, value);
        }
    }
}
