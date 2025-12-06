use alloc::{boxed::Box, slice, vec, vec::Vec};
use spin::rwlock::RwLock;

use crate::{
    arch::endian::EndianData,
    devices::device_info::{DeviceInfo, MemoryAreaInfo},
    error::MessageError,
    mm::types::{MaybeOwned, MaybeOwnedBytes, MaybeOwnedStr},
};

#[repr(C)]
#[derive(Clone, Copy)]
/// Simple range structure used to represent start/length pairs parsed from properties.
pub struct DPRange<T> {
    /// Start address (or index) of the range.
    pub start: T,
    /// Length (size) of the range.
    pub length: T,
}

// endregion
#[derive(Debug, Clone, Copy)]
/// Cell width descriptor
///
/// `address` and `size` are counts of 32-bit cells used to encode addresses
/// and sizes respectively
pub struct RegCellWidth {
    /// Number of 32-bit cells used to encode an address.
    address: u32,
    /// Number of 32-bit cells used to encode a size/length.
    size: u32,
}

/// A device tree node.
pub struct DeviceNode {
    /// Node name
    pub node_name: MaybeOwnedStr,
    /// Unit address string parsed from the node, text after `@`.
    pub unit_addr: MaybeOwnedStr,
    /// Child device nodes.
    pub subnodes: Vec<Box<DeviceNode>>,
    /// Properties attached to this node.
    pub props: Vec<DeviceProp>,
}

impl DeviceNode {
    /// Parse the `reg` property using the given `RegCellWidth` and return ranges.
    ///
    /// Returns a vector of `DPRange<u64>` with start/length pairs. Errors if the
    /// property is missing or has an unsupported format.
    pub fn get_reg<DType: EndianData<u32>>(
        &self,
        size: &RegCellWidth,
    ) -> Result<Vec<DPRange<u64>>, DeviceTreeError> {
        let arr = self.get_array::<DType>(MaybeOwned::Static("reg"))?;
        let cell_width = size.address + size.size;
        let len = arr.len() as u32;
        let data_cnt = cell_width / len; // ignore exceeding unmatchable types
        let mut res = Vec::new();
        for i in 0..data_cnt {
            let addr_idx = (i * cell_width) as usize;
            let sz_idx = (i * cell_width + size.address) as usize;
            let mut addr: u64 = arr[addr_idx].value() as u64;
            let mut sz: u64 = arr[sz_idx].value() as u64;
            if size.address == 2 {
                addr = (addr << 32) + arr[addr_idx + 1].value() as u64;
            }
            if size.size == 2 {
                sz = (sz << 32) + arr[sz_idx + 1].value() as u64;
            }
            res.push(DPRange::<u64> {
                start: addr,
                length: sz,
            });
        }
        Ok(res)
    }

    /// Return a typed pointer to the start of the property's raw data.
    pub fn get_array_as_ptr<DType: Copy>(
        &self,
        prop_name: MaybeOwnedStr,
    ) -> Result<&DType, DeviceTreeError> {
        let p = self
            .find_prop(prop_name.as_ref())
            .ok_or(DeviceTreeError::PropertyNotFound {
                prop_name: prop_name,
            })?;
        Ok(unsafe { &*((&p.raw_data[0]) as *const u8 as *const DType) })
    }

    /// Interpret the property's raw bytes as an array of `DType` entries, ignoring exceeding not-aligned bytes.
    pub fn get_array<DType: Copy>(
        &self,
        prop_name: MaybeOwnedStr,
    ) -> Result<&[DType], DeviceTreeError> {
        let p = self
            .find_prop(prop_name.as_ref())
            .ok_or(DeviceTreeError::PropertyNotFound {
                prop_name: prop_name,
            })?;
        let len = p.raw_data.len() / size_of::<DType>(); // ignore exceeding unmatchable bytes
        unsafe {
            Ok(slice::from_raw_parts(
                p.raw_data.as_ref().as_ptr() as *const DType,
                len,
            ))
        }
    }

    /// Read a value and cast it to the specific type from the property's raw bytes.
    pub fn get_value_as<TVal: Copy>(&self, name: MaybeOwnedStr) -> Result<TVal, DeviceTreeError> {
        unsafe {
            let r = *(self
                .find_prop(name.as_ref())
                .ok_or(DeviceTreeError::PropertyNotFound { prop_name: name })?
                .raw_data
                .as_ptr() as *const TVal);
            Ok(r)
        }
    }

    /// Find a property by its name
    pub fn find_prop(&self, name: &str) -> Option<&DeviceProp> {
        self.find_props([name])[0]
    }

    /// Find properties by their names
    pub fn find_props<const T: usize>(&self, name: [&str; T]) -> [Option<&DeviceProp>; T] {
        let mut res = [None; T];
        for prop in &self.props {
            let mut iter = name.iter();
            if let Some(idx) = iter.position(|&s| prop.prop_name.eq(s)) {
                res[idx] = Some(prop);
            }
        }
        res
    }

    /// Return all direct child nodes by the name
    pub fn find_nodes(&self, name: &str) -> Vec<&DeviceNode> {
        let mut res = Vec::<&DeviceNode>::new();
        for node in &self.subnodes {
            if node.node_name.eq(name) {
                res.push(node.as_ref());
            }
        }
        res
    }
}

#[derive(Debug)]
pub struct DeviceProp {
    /// Property name
    pub prop_name: MaybeOwnedStr,
    /// Raw bytes of the property data
    pub raw_data: MaybeOwnedBytes,
}

/// Validate whether the cell width is supported by this parser
fn validate_cell_size(size: &RegCellWidth) -> Result<(), DeviceTreeError> {
    if size.address > 2 || size.size > 2 {
        return Err(DeviceTreeError::UnsupportedAddressType {
            address_cells: size.address,
            size_cells: size.size,
        });
    }
    Ok(())
}

pub struct EmbeddedDeviceInfo {
    /// Memory areas discovered in the device tree.
    pub mem_area: Vec<MemoryAreaInfo>,
}

/// Trait describing a parsed device tree provider.
pub trait DeviceTree: DeviceInfo {
    /// Endian-aware 32-bit cell reader used when interpreting property payloads.
    type TDataType: EndianData<u32>;

    /// Return a reference to the `RwLock` guarding the optional root `DeviceNode`.
    ///
    /// The lock is used by default helper implementations to access the
    /// parsed tree.
    fn get_root_node_lock(&self) -> &RwLock<Option<DeviceNode>>;

    /// Read `#address-cells` and `#size-cells` from the root node
    fn get_cell_size(&self) -> Result<RegCellWidth, DeviceTreeError> {
        let guard = self.get_root_node_lock().read();
        let root_node = guard.as_ref().unwrap();
        let addr_cells = root_node
            .get_value_as::<Self::TDataType>(MaybeOwned::Static("#address-cells"))?
            .value();
        let size_cells = root_node
            .get_value_as::<Self::TDataType>(MaybeOwned::Static("#size-cells"))?
            .value();
        Ok(RegCellWidth {
            address: addr_cells,
            size: size_cells,
        })
    }

    /// Initialize basic devices.
    fn init_devices(&self) -> Result<EmbeddedDeviceInfo, DeviceTreeError> {
        let guard = self.get_root_node_lock().read();
        let root_node = guard.as_ref().unwrap();

        let cell_sz = self.get_cell_size()?;
        validate_cell_size(&cell_sz)?;

        let mem_area =
            root_node
                .find_nodes("memory")
                .iter()
                .try_fold(vec![], |mut acc, &val| {
                    val.get_reg::<Self::TDataType>(&cell_sz)?
                        .iter()
                        .for_each(|reg| {
                            acc.push(MemoryAreaInfo {
                                start: reg.start as usize,
                                length: reg.length as usize,
                            });
                        });
                    Ok(acc)
                })?;

        let info = EmbeddedDeviceInfo { mem_area: mem_area };
        Ok(info)
    }
}

#[derive(Debug)]
pub enum DeviceTreeError {
    /// Requested property was not found on the node.
    PropertyNotFound { prop_name: MaybeOwnedStr },
    /// Requested child node was not found.
    NodeNotFound { node_name: MaybeOwnedStr },
    /// The `#address-cells`/`#size-cells` value is not supported.
    UnsupportedAddressType { address_cells: u32, size_cells: u32 },
    /// A property had an invalid format when parsed.
    InvalidPropFormat {
        prop_name: MaybeOwnedStr,
        err: InvalidPropFormatError,
    },
}

#[derive(Debug)]
pub enum InvalidPropFormatError {
    /// The parsed array length did not match the expected count.
    ArrayLengthError {
        /// Human readable element type name (e.g. "u32").
        array_type: &'static str,
        /// Actual element count found.
        len: u32,
        /// Expected/factor used when validating length.
        supposed_fact: u32,
    },
}

impl MessageError for DeviceTreeError {}
