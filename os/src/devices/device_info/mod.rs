//! This module defines an abstraction to describe the device struct of the system
mod device_tree;
mod fdt;
use alloc::vec::Vec;
pub use fdt::FdtTree;

use crate::error::MessageError;

/// A section of the universal memory
#[derive(Debug)]
pub struct MemoryAreaInfo {
    /// Starting address
    pub start: usize,
    /// Length
    pub length: usize,
}

/// An abstraction to describe the device struct of the system
pub trait DeviceInfo {
    /// The error type the resolver would throw when encountered with errors
    type TError: MessageError;

    /// Initialize the device tree
    fn init(&self) -> Result<(), Self::TError>;

    /// Get all the universal memory sections
    fn get_mem_info(&self) -> Result<&Vec<MemoryAreaInfo>, Self::TError>;
}
