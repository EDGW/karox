//! This module provides functionalities to resolve a flattened device tree

use core::str::{self, Utf8Error};

use alloc::{boxed::Box, slice, string::String, vec::Vec};
use bitflags::bitflags;
use spin::{once::Once, rwlock::RwLock};

use crate::{
    arch::endian::{BigEndian32, EndianData},
    devices::device_info::{
        DeviceInfo, MemoryAreaInfo,
        device_tree::{DeviceNode, DeviceProp, DeviceTree, DeviceTreeError, EmbeddedDeviceInfo},
    },
    error::MessageError,
    mm::types::{MaybeOwned, MaybeOwnedStr},
};

/// Parser and holder for a Flattened Device Tree (FDT) blob.
///
/// `FdtTree` stores a pointer to the FDT in memory and lazily builds the
/// in-memory [DeviceNode] tree and extracted device information.
pub struct FdtTree {
    /// Raw pointer to the FDT blob interpreted as big-endian 32-bit words.
    fdt_ptr: *const BigEndian32,
    /// Root node parsed from the FDT (protected by an RwLock for concurrent access).
    pub fdt_node: RwLock<Option<DeviceNode>>,
    /// Lazily-initialized extracted device info (memory regions, devices, etc.).
    pub devices: Once<EmbeddedDeviceInfo>,
}

/// Raw on-disk Flattened Device Tree header (big-endian fields).
///
/// This maps directly to the FDT header structure; fields are stored as
/// big-endian 32-bit values and should be interpreted as `EndianData`.
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

/// Read a 32-bit big-endian word from `ptr` without advancing the pointer.
#[inline(always)]
fn peek(ptr: &mut *const BigEndian32) -> u32 {
    unsafe { (**ptr).value() }
}

/// Read a 32-bit big-endian word from `ptr` and advance the pointer by 4 bytes.
#[inline(always)]
fn read(ptr: &mut *const BigEndian32) -> u32 {
    unsafe {
        let res = (**ptr).value();
        *ptr = (*ptr).add(1);
        res
    }
}

/// Read `len` bytes starting at `ptr` and advance `ptr` to the next 4-byte aligned position.
///
/// Returns a `'static` slice that points into the original FDT memory blob.
#[inline(always)]
fn readbytes_aligned(ptr: &mut *const BigEndian32, len: u32) -> &'static [u8] {
    let s = *ptr as *const u8;
    unsafe {
        let e = s.add(len as usize);
        let mut eu = e as usize;
        let m = eu % 4;
        if m != 0 {
            eu += 4 - m;
        }
        *ptr = eu as *const BigEndian32;
        slice::from_raw_parts(s, len as usize)
    }
}

/// Advance `ptr` past zero words and NOPs to the next meaningful token.
#[inline(always)]
fn skip(ptr: &mut *const BigEndian32) {
    let mut p = peek(ptr);
    while p == 0 || p == FdtNodeType::FDT_NOP.bits {
        unsafe {
            *ptr = (*ptr).add(1);
        }
        p = peek(ptr)
    }
}

/// Read a NUL-terminated string from `ptr` and advance `ptr` to the next aligned position.
///
/// Returns a borrowed `'static` str pointing into the FDT blob, or an `Utf8Error`
/// wrapped in `FdtError` if the bytes are not valid UTF-8.
#[inline(always)]
fn readstr_aligned(ptr: &mut *const BigEndian32) -> Result<&'static str, FdtError> {
    let start = *ptr as *const u8;
    let mut end = *ptr as *const u8;
    let mut len = 0;
    unsafe {
        while *end != 0 {
            end = end.add(1);
            len += 1;
        }
        let mut end_pos = end as usize;
        let m = end_pos % 4;
        if m != 0 {
            end_pos += 4 - m;
        }
        *ptr = end_pos as *const BigEndian32;
        match str::from_utf8(slice::from_raw_parts(start, len)) {
            Ok(st) => Ok(st),
            Err(utf_err) => Err(FdtError::Utf8Error {
                err: utf_err,
                ptr_str: start,
            }),
        }
    }
}

/// Read a tag word and verify it equals `supposed`.
///
/// Returns `Ok(())` if the tag matches, otherwise returns
/// `FdtError::InvalidNodeType` with the current tag and pointer.
fn read_and_check(ptr: &mut *const BigEndian32, supposed: FdtNodeType) -> Result<(), FdtError> {
    let node_type = peek(ptr);
    read(ptr);
    if node_type != supposed.bits {
        return Err(FdtError::InvalidNodeType {
            cur_type: node_type,
            acceptable_types: supposed,
            ptr: (*ptr).clone() as *const u8,
        });
    }
    Ok(())
}

impl FdtTree {
    /// Expected FDT magic number (0xd00dfeed).
    pub const FDT_MAGIC: u32 = 0xd00dfeed;
    /// The FDT version this parser targets.
    pub const FDT_VERSION: usize = 17;
    /// The last compatible FDT version accepted by this parser.
    pub const LAST_COMP_VERSION: usize = 16;
    // region: constructor

    /// Create an `FdtTree` from a raw pointer to an FDT blob in memory.
    ///
    /// This does not validate the blob; call `validate()` before using parser
    /// helpers to ensure the header and version are acceptable.
    #[inline(always)]
    pub fn from_ptr(ptr: *const u8) -> FdtTree {
        FdtTree {
            fdt_ptr: ptr as *const BigEndian32,
            fdt_node: RwLock::new(None),
            devices: Once::new(),
        }
    }

    // endregion

    // region: helper methods:

    #[inline(always)]
    fn get_pointer8(&self, offset: u32) -> *const u8 {
        unsafe { (self.fdt_ptr as *const u8).add(offset as usize) }
    }

    #[inline(always)]
    fn get_pointer32(&self, offset: u32) -> *const BigEndian32 {
        unsafe { (self.fdt_ptr as *const u8).add(offset as usize) as *const BigEndian32 }
    }

    #[inline(always)]
    fn get_end8(&self) -> *const u8 {
        self.get_pointer8(self.get_header().totalsize.value())
    }

    // endregion

    /// Return a reference to the FDT header.
    ///
    /// Note: this only performs a pointer cast. The header contents should be
    /// validated with `validate()` before relying on the fields.
    #[inline(always)]
    pub fn get_header<'a>(&'a self) -> &'a FdtHeader {
        unsafe { &*(self.fdt_ptr as *const FdtHeader) }
    }

    /// Validate the FDT header (magic number and compatible version range).
    ///
    /// Returns `Ok(())` on success, or an `FdtError` describing the failure.
    pub fn validate(&self) -> Result<(), FdtError> {
        let header = self.get_header();
        kserial_println!(
            "header {:#x}",
            header as *const FdtHeader as *const u8 as usize
        );
        let magic = header.magic.value();

        // 1. Check the magic number
        if magic != FdtTree::FDT_MAGIC {
            return Err(FdtError::MagicError { magic: magic });
        }

        // 2. Check the fdt version. We use version 17, and the last compatible version is 16
        let version = header.version.value();
        if version < FdtTree::LAST_COMP_VERSION as u32
            || header.last_comp_version.value() > FdtTree::FDT_VERSION as u32
        {
            return Err(FdtError::VersionError { version: version });
        }
        Ok(())
    }

    /// Read a null-terminated string from the FDT string table at `offset`.
    ///
    /// Returns a borrowed `'static` str pointing into the FDT blob or an
    /// `FdtError::Utf8Error` if the bytes are not valid UTF-8.
    pub fn get_string(&self, offset: u32) -> Result<&'static str, FdtError> {
        let s = self.get_pointer8(self.get_header().off_dt_strings.value() + offset);
        let bound = self.get_end8();
        let mut e = s.clone();
        let mut len = 0;
        unsafe {
            while *e != 0 && e < bound {
                e = e.add(1);
                len += 1;
            }
            match str::from_utf8(slice::from_raw_parts(s, len)) {
                Ok(s) => Ok(s),
                Err(utf_err) => Err(FdtError::Utf8Error {
                    err: utf_err,
                    ptr_str: s,
                }),
            }
        }
    }

    /// Read consecutive property entries from the structure block and return them.
    ///
    /// Stops when a non-`FDT_PROP` tag is encountered and returns the collected props.
    fn read_props(&self, ptr: &mut *const BigEndian32) -> Result<Vec<DeviceProp>, FdtError> {
        let mut res = Vec::<DeviceProp>::new();
        loop {
            skip(ptr);
            if peek(ptr) != FdtNodeType::FDT_PROP.bits {
                break Ok(res);
            }
            read(ptr);
            let len = read(ptr);
            let name_offset = read(ptr);
            let name = MaybeOwnedStr::Static(self.get_string(name_offset)?);
            let data = MaybeOwned::<[u8]>::Static(readbytes_aligned(ptr, len));
            res.push(DeviceProp {
                prop_name: name,
                raw_data: data,
            });
        }
    }

    /// Parse a single node (name, properties and child nodes) from the structure block.
    ///
    /// Recursively parses subnodes until the matching `FDT_END_NODE` is found.
    fn read_node(&self, ptr: &mut *const BigEndian32) -> Result<DeviceNode, FdtError> {
        skip(ptr);
        read_and_check(ptr, FdtNodeType::FDT_BEGIN_NODE)?;
        let name_full = readstr_aligned(ptr)?;
        let node_name: &str;
        let unit_addr: &str;
        let at_idx = name_full.find('@');
        match at_idx {
            Some(idx) => {
                node_name = &name_full[0..idx];
                unit_addr = &name_full[idx + 1..name_full.len()]
            }
            None => {
                node_name = name_full;
                unit_addr = "";
            }
        }
        let props = self.read_props(ptr)?;
        let mut subnodes = Vec::<Box<DeviceNode>>::new();
        loop {
            skip(ptr);
            let nodetype = peek(ptr);
            if nodetype == FdtNodeType::FDT_BEGIN_NODE.bits {
                subnodes.push(Box::new(self.read_node(ptr)?));
            } else if nodetype == FdtNodeType::FDT_END_NODE.bits {
                read(ptr);
                break;
            } else {
                return Err(FdtError::InvalidNodeType {
                    cur_type: nodetype,
                    acceptable_types: FdtNodeType::FDT_BEGIN_NODE,
                    ptr: *ptr as *const u8,
                });
            }
        }

        Ok(DeviceNode {
            node_name: MaybeOwnedStr::Static(node_name),
            unit_addr: MaybeOwnedStr::Static(unit_addr),
            props: props,
            subnodes: subnodes,
        })
    }

    /// Parse the entire structure block and store the root [DeviceNode].
    ///
    /// After calling this the in-memory node tree is available via [Self::fdt_node].
    pub fn load_nodes(&self) -> Result<(), FdtError> {
        let mut ptr = self.get_pointer32(self.get_header().off_dt_struct.value());
        let node = self.read_node(&mut ptr)?;
        skip(&mut ptr);
        read_and_check(&mut ptr, FdtNodeType::FDT_END)?;
        *(self.fdt_node.write()) = Some(node);
        Ok(())
    }

    /// Debug helper to print the node tree in a compact, non-standard format.
    #[allow(unused)]
    pub fn print_nodes(&self, node: &DeviceNode, tab: usize) {
        let tabstr = String::from("\t").repeat(tab);
        kserial_print!("{:} -- Node {:} [", tabstr, node.node_name);
        for nd in &node.props {
            kserial_print!("@'{:}'={:?}, ", nd.prop_name, nd.raw_data);
        }
        kserial_print!("]\n");
        for subn in &node.subnodes {
            self.print_nodes(subn, tab + 1);
        }
    }

    /// Return the parsed [EmbeddedDeviceInfo] if the device tree has been initialized.
    pub fn get_device_info(&self) -> Result<&EmbeddedDeviceInfo, String> {
        self.devices
            .get()
            .ok_or(String::from("Device tree not initialized."))
    }
}

impl DeviceInfo for FdtTree {
    type TError = FdtError;

    /// Initialize the FDT parser and extract device info.
    ///
    /// This validates the header, parses the node tree and populates the
    /// extracted [EmbeddedDeviceInfo] used by the rest of the kernel.
    fn init(&self) -> Result<(), FdtError> {
        self.validate()?;
        self.load_nodes()?;
        let dev = self
            .init_devices()
            .map_err(|err| FdtError::DeviceTreeError { err: err })?;
        self.devices.call_once(|| dev);
        Ok(())
    }

    /// Return memory area info parsed from the device tree.
    fn get_mem_info(&self) -> Result<&Vec<MemoryAreaInfo>, FdtError> {
        Ok(&self
            .devices
            .get()
            .ok_or(FdtError::NotInitializedError)?
            .mem_area)
    }
}

impl DeviceTree for FdtTree {
    type TDataType = BigEndian32;

    fn get_root_node_lock(&self) -> &RwLock<Option<DeviceNode>> {
        &self.fdt_node
    }
}

#[derive(Debug)]
pub enum FdtError {
    /// Header magic number did not match the expected FDT magic.
    MagicError { magic: u32 },
    /// FDT version is out of supported/compatible range.
    VersionError { version: u32 },
    /// The FDT contains a string that is not valid UTF-8.
    Utf8Error {
        err: Utf8Error,
        /// Pointer to the offending string in the blob.
        ptr_str: *const u8,
    },
    /// Packed [DeviceTreeError]
    DeviceTreeError { err: DeviceTreeError },
    /// Encountered an unexpected node tag while parsing the structure block.
    InvalidNodeType {
        cur_type: u32,
        acceptable_types: FdtNodeType,
        ptr: *const u8,
    },
    /// Device tree parsing or initialization has not been performed yet.
    NotInitializedError,
}

impl MessageError for FdtError {}
