//! This module provides a simple linear memory pool.
//!
//! # Overview
//!
//! `LinearPool` is a minimal bump allocator used to allocate permanent,
//! non-freeable memory regions during early boot or low-level initialization.
//!
//! It behaves like a stack: memory is always allocated from the top, and the
//! pool never supports freeing or rewinding.  

/// A linear, bump-pointer–style memory pool.
pub struct LinearPool{
    /// The starting address of the memory pool.
    pub start: *mut u8,
    /// The current top address of the pool.  
    /// New allocations are taken from this pointer.
    pub top: *mut u8
}

impl LinearPool{
    /// Create a [LinearPool] that begins at the given base address.
    ///
    /// The pool initially has both `start` and `top` set to `start`.
    pub fn from(start: *mut u8) -> LinearPool{
        LinearPool { start, top: start }
    }

    
    /// Allocate a memory region of `size` bytes from the pool and return its base pointer.
    ///
    /// This function simply moves the bump pointer forward to allocate a memory space and returns a pointer to the allocated space
    /// The caller is responsible for ensuring:
    /// * The pool has enough space.
    /// * Alignment requirements are satisfied, if needed.
    ///
    /// No memory is ever freed.
    pub fn take(&mut self, size: usize) -> *mut u8{
        let res = self.top;
        unsafe{
            self.top = self.top.add(size);
        }
        res
    }
}