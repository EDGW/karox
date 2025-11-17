//! This module provides a linear memory pool, which functions like a stack, and is only used for allocate
//! permanent memory blocks in specific size

/// A linear pool struct
pub struct LinearPool{
    /// the starting point of the pool
    pub start: *mut u8,
    /// the pool top
    pub top: *mut u8
}

impl LinearPool{
    /// Create a linear pool from a specific address
    pub fn from(start: *mut u8) -> LinearPool{
        LinearPool { start, top: start }
    }

    /// Take a memory area of a specific size and return the pointer
    pub fn take(&mut self, size: usize) -> *mut u8{
        let res = self.top;
        unsafe{
            self.top = self.top.add(size);
        }
        res
    }
}