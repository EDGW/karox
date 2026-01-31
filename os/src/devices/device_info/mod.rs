//! This module defines an abstraction to describe the device struct of the system
mod device_tree;
mod fdt;
use core::{fmt::Debug, ops::Range};

use alloc::vec::Vec;
pub use fdt::FdtTree;

pub type MemoryAreaInfo = Range<usize>;

pub struct HartInfo {
    pub hart_id: usize,
}

/// An abstraction to describe the device struct of the system
pub trait DeviceInfo {
    /// The error type the resolver would throw when encountered with errors
    type TError: Debug;

    /// Initialize the device tree
    fn init(&self) -> Result<(), Self::TError>;

    /// Get all the general memory sections
    fn get_mem_info(&self) -> Result<&Vec<MemoryAreaInfo>, Self::TError>;

    /// Get all logical harts
    fn get_hart_info(&self) -> Result<&Vec<HartInfo>, Self::TError>;
}
