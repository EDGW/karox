use core::{mem::swap, ops::Range};

use crate::{
    fdt::{FdtHeader, FdtNodeType, ReservedMemoryEntry},
    node::{DeviceTree, Node, NodeType},
    prop::Property,
};
use alloc::{boxed::Box, slice, vec, vec::Vec};
use utils::{
    endian::{BigEndian32, EndianData},
    num::AlignableTo,
};

pub struct FdtReader {
    header_ptr: *const BigEndian32,
    cursor: *const BigEndian32,
    nodes: Vec<Node>,
}

/// Basic Reader Functions
impl FdtReader {
    /// Read a 32-bit big-endian word from `ptr` without advancing the pointer.
    #[inline(always)]
    fn peek_u32(&mut self) -> u32 {
        unsafe { (*self.cursor).value() }
    }

    /// Advance the pointer by 4 bytes.
    #[inline(always)]
    fn advance(&mut self) {
        unsafe {
            self.cursor = self.cursor.add(1);
        }
    }

    /// Advance the pointer by specific bytes and align the pointer to 4 bytes.
    #[inline(always)]
    fn advance_bytes_aligned(&mut self, step: usize) {
        self.cursor = (self.cursor as usize + step).align_up(4) as *const BigEndian32
    }

    /// Read a 32-bit big-endian word from `ptr` and advance the pointer by 4 bytes.
    #[inline(always)]
    fn read_u32(&mut self) -> u32 {
        let res = self.peek_u32();
        self.advance();
        res
    }
    /// Read `len` bytes starting at `ptr` and advance `ptr` to the next 4-byte aligned position.
    ///
    /// Returns a `'static` slice that points into the original FDT memory blob.
    #[inline(always)]
    fn readbytes_aligned(&mut self, len: usize) -> &'static [u8] {
        let res = unsafe { slice::from_raw_parts(self.cursor as *const u8, len) };
        self.advance_bytes_aligned(len);
        res
    }

    /// Advance `ptr` past zero words and NOPs to the next meaningful token.
    #[inline(always)]
    fn skip(&mut self) {
        let mut p = self.peek_u32();
        while p == 0 || p == FdtNodeType::FDT_NOP.bits {
            self.advance();
            p = self.peek_u32()
        }
    }

    /// Read a NUL-terminated string from `ptr` and advance `ptr` to the next aligned position.
    ///
    /// Returns a borrowed `'static` str pointing into the FDT blob.
    #[inline(always)]
    fn readstr_aligned(&mut self) -> &'static str {
        let start = self.cursor as *const u8;
        let mut end = self.cursor as *const u8;
        let mut len = 0;
        unsafe {
            while *end != 0 {
                end = end.add(1);
                len += 1;
            }
            self.cursor = (end as usize).align_up(4) as *const BigEndian32;
            str::from_utf8_unchecked(slice::from_raw_parts(start, len))
        }
    }

    /// Read a tag word and verify it equals `supposed`.
    ///
    /// Returns `Ok(())` if the tag matches, otherwise returns
    /// `FdtError::InvalidNodeType` with the current tag and pointer.
    fn read_and_check(&mut self, supposed: FdtNodeType) -> Result<(), FdtError> {
        let node_type = self.read_u32();
        if node_type != supposed.bits {
            return Err(FdtError::InvalidNodeType {
                node_type: node_type as usize,
                cursor: self.cursor as usize,
            });
        }
        Ok(())
    }
}

impl FdtReader {
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
    pub fn new(ptr: *const u8) -> FdtReader {
        FdtReader {
            cursor: ptr as *const BigEndian32,
            header_ptr: ptr as *const BigEndian32,
            nodes: vec![],
        }
    }

    // endregion

    // region: helper methods:

    #[inline(always)]
    fn get_pointer8(&self, offset: usize) -> *const u8 {
        unsafe { (self.header_ptr as *const u8).add(offset) }
    }

    #[inline(always)]
    fn get_pointer32(&self, offset: usize) -> *const BigEndian32 {
        unsafe { (self.header_ptr as *const u8).add(offset) as *const BigEndian32 }
    }

    #[inline(always)]
    fn get_end8(&self) -> *const u8 {
        self.get_pointer8(self.get_header().totalsize.value() as usize)
    }

    // endregion

    /// Return a reference to the FDT header.
    ///
    /// Note: this only performs a pointer cast. The header contents should be
    /// validated with `validate()` before relying on the fields.
    #[inline(always)]
    pub fn get_header<'a>(&'a self) -> &'a FdtHeader {
        unsafe { &*(self.header_ptr as *const FdtHeader) }
    }

    /// Validate the FDT header (magic number and compatible version range).
    ///
    /// Returns `Ok(())` on success, or an `FdtError` describing the failure.
    pub fn validate(&self) -> Result<(), FdtError> {
        let header = self.get_header();
        let magic = header.magic.value();

        // 1. Check the magic number
        if magic != Self::FDT_MAGIC {
            return Err(FdtError::InvalidMagic {
                magic: magic as usize,
            });
        }

        // 2. Check the fdt version. We use version 17, and the last compatible version is 16
        let version = header.version.value();
        if version < Self::LAST_COMP_VERSION as u32
            || header.last_comp_version.value() > Self::FDT_VERSION as u32
        {
            return Err(FdtError::IncompatibleVersion {
                version: version as usize,
            });
        }
        Ok(())
    }

    /// Read a null-terminated string from the FDT string table at `offset`.
    ///
    /// Returns a borrowed `'static` str pointing into the FDT blob or an
    /// `FdtError::Utf8Error` if the bytes are not valid UTF-8.
    pub fn get_string(&self, offset: usize) -> &'static str {
        let s = self.get_pointer8(self.get_header().off_dt_strings.value() as usize + offset);
        let bound = self.get_end8();
        let mut e = s.clone();
        let mut len = 0;
        unsafe {
            while *e != 0 && e < bound {
                e = e.add(1);
                len += 1;
            }
            str::from_utf8_unchecked(slice::from_raw_parts(s, len))
        }
    }

    /// Read consecutive property entries from the structure block and return them.
    ///
    /// Stops when a non-`FDT_PROP` tag is encountered and returns the collected props.
    fn read_props(&mut self) -> Result<Vec<Property>, FdtError> {
        let mut res = Vec::<Property>::new();
        loop {
            self.skip();
            if self.peek_u32() != FdtNodeType::FDT_PROP.bits {
                break Ok(res);
            }
            self.read_u32();
            let len = self.read_u32() as usize;
            let name_offset = self.read_u32() as usize;
            let name = Box::from(self.get_string(name_offset));
            let data = Box::from(self.readbytes_aligned(len));
            res.push(Property { name, data });
        }
    }

    /// Parse a single node (name, properties and child nodes) from the structure block without setting its parent.
    ///
    /// Recursively parses subnodes until the matching `FDT_END_NODE` is found.
    fn read_node(&mut self) -> Result<usize, FdtError> {
        self.skip();
        self.read_and_check(FdtNodeType::FDT_BEGIN_NODE)?;
        let full_name = self.readstr_aligned();
        let node_name: &str;
        let unit_addr: &str;
        let at_idx = full_name.find('@');
        match at_idx {
            Some(idx) => {
                node_name = &full_name[0..idx];
                unit_addr = &full_name[idx + 1..full_name.len()]
            }
            None => {
                node_name = full_name;
                unit_addr = "";
            }
        }
        let props = self.read_props()?;
        let mut children = vec![];
        loop {
            self.skip();
            let nodetype = self.peek_u32();
            if nodetype == FdtNodeType::FDT_BEGIN_NODE.bits {
                children.push(self.read_node()?);
            } else if nodetype == FdtNodeType::FDT_END_NODE.bits {
                self.advance();
                break;
            } else {
                return Err(FdtError::InvalidNodeType {
                    node_type: nodetype as usize,
                    cursor: self.cursor as usize,
                });
            }
        }
        let id = self.nodes.len();
        let node = Node {
            node_id: id,
            parent_id: 0,
            full_name: Box::from(full_name),
            node_name: Box::from(node_name),
            unit_addr: Box::from(unit_addr),
            children,
            props,
            node_type: NodeType::Device,
        };
        self.nodes.push(node);
        Ok(id)
    }

    fn set_parent(&mut self, node_id: usize) {
        for child_idx in 0..self.nodes[node_id].children.len() {
            let sub_id = self.nodes[node_id].children[child_idx];
            self.nodes[sub_id].parent_id = node_id;
            self.set_parent(sub_id);
        }
    }

    /// Get the memory reservation map. The reserved memory block is not aligned.
    ///
    /// **The reserved memory block are not promised to be not overlapped**
    ///
    /// Automatically add the fdt itself to the reservation block if it's not in the reservation block.
    fn get_mem_rsv_map(&self) -> Result<Vec<Range<usize>>, FdtError> {
        let header = self.get_header();
        let mut ptr =
            self.get_pointer8(header.off_mem_rsvmap.value() as usize) as *const ReservedMemoryEntry;
        let mut res = Vec::new();

        // self
        let self_range = Range {
            start: (self.header_ptr as usize),
            end: (self.header_ptr as usize + header.totalsize.value() as usize),
        };

        // enumerate
        unsafe {
            let mut block = *ptr;
            let mut addr = block.addr.value() as usize;
            let mut size = block.size.value() as usize;
            while addr != 0 || size != 0 {
                res.push(Range {
                    start: addr,
                    end: (size + addr),
                });

                ptr = ptr.add(1);
                block = *ptr;
                addr = block.addr.value() as usize;
                size = block.size.value() as usize;
            }
        }
        res.push(self_range);

        Ok(res)
    }

    fn read_internal(&mut self) -> Result<DeviceTree, FdtError> {
        self.cursor = self.get_pointer32(self.get_header().off_dt_struct.value() as usize);
        let root_id = self.read_node()?;
        self.set_parent(root_id);
        self.nodes[root_id].parent_id = root_id;
        self.skip();
        self.read_and_check(FdtNodeType::FDT_END)?;

        let mut tree = DeviceTree {
            root_id,
            container: vec![],
            mem_rsv_map: self.get_mem_rsv_map()?,
        };
        swap(&mut self.nodes, &mut tree.container);
        if let Some(node) = tree.get_node_mut("/aliases") {
            node.node_type = NodeType::Description;
        }
        tree.get_nodes_mut("/memory", |node| {
            node.node_type = NodeType::Description;
        });
        if let Some(node) = tree.get_node_mut("/reserved-memory") {
            node.node_type = NodeType::Description;
        }
        if let Some(node) = tree.get_node_mut("/chosen") {
            node.node_type = NodeType::Description;
        }
        Ok(tree)
    }

    /// Parse the entire structure block and store the root [DeviceNode].
    ///
    /// After calling this, the in-memory node tree is available via [Self::fdt_node].
    ///
    /// All strings and byte-array data will be **copied** to [Self::fdt_node], and the raw data of fdt can be safely wiped.
    pub fn read(&mut self) -> Result<DeviceTree, FdtError> {
        match self.read_internal() {
            Ok(res) => Ok(res),
            Err(err) => {
                self.cursor = self.header_ptr;
                self.nodes.clear();
                Err(err)
            }
        }
    }
}

#[derive(Debug)]
pub enum FdtError {
    InvalidNodeType { node_type: usize, cursor: usize },
    InvalidMagic { magic: usize },
    IncompatibleVersion { version: usize },
}
