use core::str::{self, Utf8Error};

use alloc::{boxed::Box, slice, string::String, vec::Vec};
use bitflags::bitflags;
use spin::{once::Once, rwlock::RwLock};

use crate::{
    arch::endian::{BigEndian32, EndianData},
    devices::device_tree::{
        DeviceTree, MemoryAreaInfo,
        device_tree::{DeviceNode, DeviceProp, DeviceTreeError, EmbeddedDeviceInfo},
    },
    error::MessageError,
    mm::types::{MaybeOwned, MaybeOwnedStr},
};

pub struct FdtTree {
    fdt_ptr: *const BigEndian32,
    pub fdt_node: RwLock<Option<DeviceNode>>,
    pub devices: Once<EmbeddedDeviceInfo>,
}

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
    pub struct FdtNodeType : u32{
        const FDT_BEGIN_NODE  = 0x01;
        const FDT_END_NODE    = 0x02;
        const FDT_PROP        = 0x03;
        const FDT_NOP         = 0x04;
        const FDT_END         = 0x09;
    }
}

/// Read a word without advcancing the pointer
#[inline(always)]
fn peek(ptr: &mut *const BigEndian32) -> u32 {
    unsafe { (**ptr).value() }
}

/// Read a word and advance the pointer
#[inline(always)]
fn read(ptr: &mut *const BigEndian32) -> u32 {
    unsafe {
        let res = (**ptr).value();
        *ptr = (*ptr).add(1);
        res
    }
}

/// Read `len` bytes from the pointer, and automatically move the pointer to keep it 4-byte-aligned.
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

/// Advancing the pointer to the first value that is not `0` or [FdtNodeType::FDT_NOP]
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

/// Read a null terminated string from the pointer, and automatically move the pointer to keep it 4-byte-aligned.
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

/// Read a word as a node type and check whether the node type equal the supposed type
///
/// Returns nothing if check succeeds, or throws a [FdtError::InvalidNodeType].
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
    pub const FDT_MAGIC: u32 = 0xd00dfeed;
    pub const FDT_VERSION: usize = 17;
    pub const LAST_COMP_VERSION: usize = 16;
    // region: constructor

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

    /// Get the fdt header pointer
    ///
    /// **The function only performs pointer type conversion and does not guarantee the validity of the returned data.**
    #[inline(always)]
    pub fn get_header<'a>(&'a self) -> &'a FdtHeader {
        unsafe { &*(self.fdt_ptr as *const FdtHeader) }
    }

    /// Check whether the fdt header is valid, including checking magic number and compatible version.
    ///
    /// Returns nothing if the check succeeds, or throws a [FdtError]
    pub fn validate(&self) -> Result<(), FdtError> {
        let header = self.get_header();
        kserial_println!("header {:#x}",header as *const FdtHeader as *const u8 as usize);
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

    /// Read a null-terminated string in the string table.
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

    /// Read all property nodes
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

    /// Read a node
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

    /// Load all the nodes and save the root node
    pub fn load_nodes(&self) -> Result<(), FdtError> {
        let mut ptr = self.get_pointer32(self.get_header().off_dt_struct.value());
        let node = self.read_node(&mut ptr)?;
        skip(&mut ptr);
        read_and_check(&mut ptr, FdtNodeType::FDT_END)?;
        *(self.fdt_node.write()) = Some(node);
        Ok(())
    }

    /// Print all the nodes in a non-standard format, only for debug use
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

    pub fn get_device_info(&self) -> Result<&EmbeddedDeviceInfo, String> {
        self.devices
            .get()
            .ok_or(String::from("Device tree not initialized."))
    }
}

impl DeviceTree for FdtTree {
    type TError = FdtError;
    type TDataType = BigEndian32;

    fn init(&self) -> Result<(), FdtError> {
        self.validate()?;
        self.load_nodes()?;
        let dev = self.init_devices().map_err(|err| FdtError::DeviceTreeError { err: err })?;
        self.devices.call_once(||dev);
        Ok(())
    }

    fn get_root_node_lock(&self) -> &RwLock<Option<DeviceNode>> {
        &self.fdt_node
    }

    fn get_mem_info(&self) -> Result<&Vec<MemoryAreaInfo>, FdtError> {
        Ok(&self
            .devices
            .get()
            .ok_or(FdtError::NotInitializedError)?
            .mem_area)
    }
}

#[derive(Debug)]
pub enum FdtError {
    MagicError {
        magic: u32,
    },
    VersionError {
        version: u32,
    },
    Utf8Error {
        err: Utf8Error,
        ptr_str: *const u8,
    },
    DeviceTreeError {
        err: DeviceTreeError,
    },
    InvalidNodeType {
        cur_type: u32,
        acceptable_types: FdtNodeType,
        ptr: *const u8,
    },
    NotInitializedError,
}

impl MessageError for FdtError {}
