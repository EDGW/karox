//! This module provide functions to resolve the memory struct block

use core::ops::Range;

use config::mm::endian::BigEndian64;

use crate::structure::{NodeInfo};

/// Structure of the `reg` property of a memory block, used to desribe the valid memory range.
#[derive(Clone, Copy)]
pub struct MemoryReg{
    /// Base Address of the available memory
    pub base_addr:  BigEndian64,
    /// Length of the available memory
    pub size:       BigEndian64
}
/// Get the [MemoryReg] info from a valid memory node, or returns [Err] if the node is invalid.
pub fn get_memory_range(node: &NodeInfo)-> Result<Range<u64>,&str>{
    unsafe{
        if node.get_basic_name() != "memory" || node.get_prop("device_type")?.value_as_str() != "memory" {
            return Err("Malformed memory node: node name and device_type validation failed.");
        }
        let reg = node.get_prop("reg")?.value.as_ptr() as *const MemoryReg;
        Ok(Range { start: (*reg).base_addr.value(), end: (*reg).base_addr.value() + (*reg).size.value() })
    }
}