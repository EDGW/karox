//! This module provide some functions to resolve the device tree.
//! # Brief Introduction
//! This module is designed to resolve the flattened device tree
//! 
//! [FdtPtr] represents a flattened device tree, and actually it is a packed pointer to the device tree block in the memory.
//! 
//! [FdtHeader] provides the structure for the header section of the FDT, 
//! but, as designed, this section should only be used internally to validate the structure and locate different sections.
//! 
//! [ReservedMemoryBlock] is the structure for the blocks in the reserved memory list, use [FdtPtr::enumerate_rsvmem] to enumerate it.

#![no_std]
#![no_main]
#![deny(missing_docs)]
#![deny(warnings)]
#![allow(dead_code)]

pub mod structure;

use config::mm::endian::{BigEndian32, BigEndian64};
use mm::linear_pool::LinearPool;

use crate::structure::{NodeInfo, read_node};

/// The unique valid value for [FdtHeader::magic] field
pub const FDT_MAGIC:u32 = 0xd00dfeed;

/// The Pointer to a Flattened Device Tree
pub struct FdtPtr{
    /// The pointer to the device tree blob structure
    pub dtb: *mut u8,
}

impl FdtPtr{

    /// Get [FdtPtr] struct from static address
    pub fn from_addr(dtb_addr:usize) -> FdtPtr{
        FdtPtr { dtb: dtb_addr as *mut u8 }
    }

    /// Check the magic number
    pub fn validate(&self) -> Result<(), &str>{
        unsafe{
            if (*(self.get_header())).magic.value() == FDT_MAGIC { Ok(()) }
            else { Err("Invalid FDT structure: magic number validation failed.") }
        }
    }

    /// Get the [FdtHeader] pointer
    pub fn get_header(&self) -> *const FdtHeader{
        self.dtb as *mut FdtHeader
    }

    /// Get the first [ReservedMemoryBlock] pointer
    pub fn get_rsvmem_ptr(&self) -> *const ReservedMemoryBlock{
        unsafe{
            self.dtb.add((*(self.get_header())).mem_rsvmap_offset.value() as usize)
                as *const ReservedMemoryBlock
        }
    }

    /// Enumerate every reserved memory blocks
    pub fn enumerate_rsvmem(&self, handler: fn(rsv_block: ReservedMemoryBlock)){
        unsafe{
            let mut ptr = self.get_rsvmem_ptr();
            while (*ptr).addr.value() != 0 && (*ptr).length.value() != 0{
                handler(*ptr.clone());
                ptr = ptr.add(1);
            }
        }
    }
    
    /// Get the total size of this FDT
    pub fn size(&self) -> usize{
        unsafe{
            (*(self.get_header())).totalsize.value() as usize
        }
    }

    /// Get the pointer to the end of the FDT
    pub fn end(&self) -> *const BigEndian32{
        unsafe{
            self.dtb.add(self.size()) as *const BigEndian32
        }
    }

    /// Get the pointer to the struct area
    pub fn get_struct_start(&self) -> *const BigEndian32{
        unsafe{
            self.dtb.add((*(self.get_header())).dt_struct_offset.value() as usize) as *const BigEndian32
        }
    }

    /// Load the the FDT to the linear pool and return the root node pointer
    pub fn load(&self, node_pool: &mut LinearPool) -> Result<&NodeInfo,&str>{
        let mut ptr = self.get_struct_start();
        let st = node_pool.start as *mut NodeInfo;
        let end = read_node(&mut ptr, self.end(), st)?;
        let size = end as usize - node_pool.start as usize;
        node_pool.take(size);
        unsafe{
            Ok(&*(st as *const NodeInfo))
        }
    }


}

/// The Header Struct of the FDT
pub struct FdtHeader {
    /// The magic number of the FDT
    pub magic:              BigEndian32,
    /// The total size in memory
    pub totalsize:          BigEndian32,
    /// The offset of the stucture table
    pub dt_struct_offset:   BigEndian32,
    /// The offset of the string table
    pub dt_strings_offset:  BigEndian32,
    /// The offset of the memory map
    pub mem_rsvmap_offset:  BigEndian32,
    /// The version
    pub version:            BigEndian32,
    /// Last compatible version
    pub last_comp_version:  BigEndian32,
    /// The boot cpuid
    pub boot_cpuid:         BigEndian32,
    /// The string table size
    pub dt_strings_sz:      BigEndian32,
    /// The struct table size
    pub dt_struct_sz:       BigEndian32
}

/// The structure representing a reserved memory block
/// 
/// These blocks are presented in FDT sequently,
/// with a block with both [ReservedMemoryBlock::addr] and [ReservedMemoryBlock::length] fields set to 0 as its termination
#[derive(Clone, Copy)]
pub struct ReservedMemoryBlock{
    /// The starting address of the reserved memory
    pub addr:   BigEndian64,
    /// The length of the reserved memory
    pub length: BigEndian64
}
