//! This module provides functionalities to resolve a flattened device tree

use bitflags::bitflags;
use utils::endian::{BigEndian32, BigEndian64};

pub mod reader;

/// Raw Flattened Device Tree header (big-endian fields).
///
/// This maps directly to the FDT header structure; fields are stored as
/// big-endian 32-bit values and should be interpreted as `EndianData`.
#[repr(C)]
#[derive(Debug)]
pub struct FdtHeader {
    pub magic: BigEndian32,
    pub totalsize: BigEndian32,
    pub off_dt_struct: BigEndian32,
    pub off_dt_strings: BigEndian32,
    pub off_mem_rsvmap: BigEndian32,
    pub version: BigEndian32,
    pub last_comp_version: BigEndian32,
    pub boot_cpuid_phys: BigEndian32,
    pub size_dt_strings: BigEndian32,
    pub size_dt_struct: BigEndian32,
}

/// Flattened Reserved Memory Entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ReservedMemoryEntry {
    pub addr: BigEndian64,
    pub size: BigEndian64,
}

bitflags! {
    /// Type tags found in the FDT structure block.
    pub struct FdtNodeType : u32{
        /// Begin a node (followed by its name string)
        const FDT_BEGIN_NODE  = 0x01;
        /// End a node
        const FDT_END_NODE    = 0x02;
        /// A property entry (length, nameoff, data)
        const FDT_PROP        = 0x03;
        /// No-op padding word
        const FDT_NOP         = 0x04;
        /// End of the structure block
        const FDT_END         = 0x09;
    }
}
