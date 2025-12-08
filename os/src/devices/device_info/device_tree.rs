use core::{any::type_name, fmt::Debug};

use alloc::{boxed::Box, slice, vec, vec::Vec};
use spin::rwlock::RwLock;

use crate::{
    arch::endian::EndianData,
    devices::device_info::{DeviceInfo, MemoryAreaInfo},
    error::MessageError,
    mm::types::{MaybeOwned, MaybeOwnedBytes, MaybeOwnedStr},
    utils::range::Range,
};

/// Cell width descriptor
///
/// `address` and `size` are counts of 32-bit cells used to encode addresses
/// and sizes respectively
#[derive(Debug, Clone, Copy)]
pub struct RegCellWidth {
    /// Number of 32-bit cells used to encode an address.
    pub address: usize,
    /// Number of 32-bit cells used to encode a size/length.
    pub size: usize,
}

/// A device tree node.
#[derive(Debug)]
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
    /// Parse the `reg` property using the given [RegCellWidth] and return ranges.
    ///
    /// **If values of the given [RegCellWidth] is bigger than 2, the higher parts are ignored.**
    ///
    /// Returns a vector of [Range<usize>] with start/length pairs. Errors if the
    /// property is missing or has an unsupported format.
    pub fn get_reg<DType: EndianData<u32>>(
        &self,
        size: &RegCellWidth,
    ) -> Result<Vec<Range<usize>>, DeviceTreeError> {
        let raw_array = self.get_array::<DType>(MaybeOwned::Static("reg"))?;
        let cell_width = size.address + size.size;
        let raw_length = raw_array.len();
        if raw_length % cell_width != 0 {
            return Err(DeviceTreeError::InvalidPropFormat {
                prop_name: MaybeOwned::Static("reg"),
                err: InvalidPropFormatError::ArrayLengthError {
                    array_type: type_name::<[DType]>(),
                    len: raw_length,
                    supposed_fact: cell_width,
                },
            });
        }

        let data_cnt = raw_length / cell_width;
        let mut res = Vec::new();
        for i in 0..data_cnt {
            let addr_idx = (i * cell_width) as usize;
            let sz_idx = (i * cell_width + size.address) as usize;

            let mut addr: usize = raw_array[addr_idx].value() as usize;
            let mut sz: usize = raw_array[sz_idx].value() as usize;

            if size.address >= 2 {
                addr = (addr << 32) + raw_array[addr_idx + 1].value() as usize;
            }
            if size.size == 2 {
                sz = (sz << 32) + raw_array[sz_idx + 1].value() as usize;
            }

            res.push(Range::<usize> {
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
        let prop = self
            .find_prop(prop_name.as_ref())
            .ok_or(DeviceTreeError::PropertyNotFound {
                prop_name: prop_name,
            })?;
        let raw_length = prop.raw_data.len();

        let cell_width = size_of::<DType>();
        if raw_length % cell_width != 0 {
            return Err(DeviceTreeError::InvalidPropFormat {
                prop_name: MaybeOwned::Static("reg"),
                err: InvalidPropFormatError::ArrayLengthError {
                    array_type: type_name::<[u8]>(),
                    len: raw_length,
                    supposed_fact: cell_width,
                },
            });
        }
        let data_cnt = raw_length / cell_width;

        unsafe {
            Ok(slice::from_raw_parts(
                prop.raw_data.as_ref().as_ptr() as *const DType,
                data_cnt,
            ))
        }
    }

    /// Read a value and cast it to the specific type from the property's raw bytes.
    pub fn get_value_as<TVal: Copy>(&self, name: MaybeOwnedStr) -> Result<TVal, DeviceTreeError> {
        unsafe {
            let res = *(self
                .find_prop(name.as_ref())
                .ok_or(DeviceTreeError::PropertyNotFound { prop_name: name })?
                .raw_data
                .as_ptr() as *const TVal);
            Ok(res)
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

/// A property pair of the [DeviceNode]
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

/// This struct stored the information of some embedded devices like memory and CPUs.
pub struct EmbeddedDeviceInfo {
    /// General Memory Area
    ///
    /// **Static Spaces Used by kernel/BIOS/SBI/MMIO/FDT is marked as not general is not included**
    pub general_mem: Vec<MemoryAreaInfo>,
}

/// Trait describing a parsed device tree provider.
pub trait DeviceTree: DeviceInfo {
    /// Endian-aware 32-bit cell reader used when interpreting property payloads.
    type TDataType: EndianData<u32>;

    /// Convert the [DeviceTreeError] to [DeviceInfo::TError].
    fn wrap_error(err: DeviceTreeError) -> Self::TError;

    /// Return a reference to the `RwLock` guarding the optional root `DeviceNode`.
    ///
    /// The lock is used by default helper implementations to access the
    /// parsed tree.
    fn get_root_node_lock(&self) -> Result<&RwLock<Option<DeviceNode>>, Self::TError>;

    /// Read `#address-cells` and `#size-cells` from the root node
    fn get_cell_size(&self) -> Result<RegCellWidth, Self::TError> {
        let guard = self.get_root_node_lock()?.read();
        let root_node = guard
            .as_ref()
            .ok_or(DeviceTreeError::NotInitializedError)
            .map_err(Self::wrap_error)?;

        let addr_cells = root_node
            .get_value_as::<Self::TDataType>(MaybeOwned::Static("#address-cells"))
            .map_err(Self::wrap_error)?
            .value();
        let size_cells = root_node
            .get_value_as::<Self::TDataType>(MaybeOwned::Static("#size-cells"))
            .map_err(Self::wrap_error)?
            .value();

        Ok(RegCellWidth {
            address: addr_cells as usize,
            size: size_cells as usize,
        })
    }

    /// Remove the reserved areas from a memory area given in a [DeviceNode], and add it to the result list representing all general memory areas.
    fn add_mem_to_general<'a>(
        &self,
        res: &'a mut Vec<MemoryAreaInfo>,
        mem_node: &DeviceNode,
        reserved: &Vec<Range<usize>>,
        cell_sz: &RegCellWidth,
    ) -> Result<(), Self::TError> {
        mem_node
            .get_reg::<Self::TDataType>(cell_sz)
            .map_err(Self::wrap_error)?
            .iter()
            .for_each(|reg| {
                let mut result = Vec::new();
                result.push(*reg);
                let mut temp = Vec::new();
                // In most common senses, there are only a few regs in memory map and reservation map,
                // so directly enumerating them is quicker
                for rsv_area in reserved {
                    for r in &result {
                        let areas = *r - *rsv_area;
                        if let Some(area) = areas[0] {
                            temp.push(area);
                        }
                        if let Some(area) = areas[1] {
                            temp.push(area);
                        }
                    }
                    result.clear();
                    result.append(&mut temp);
                }
                for item in result {
                    res.push(MemoryAreaInfo {
                        start: item.start as usize,
                        length: item.length as usize,
                    });
                }
            });
        Ok(())
    }

    /// Calculate the general memory area
    fn calc_mem_area(
        &self,
        root_node: &DeviceNode,
        cell_sz: &RegCellWidth,
    ) -> Result<Vec<MemoryAreaInfo>, Self::TError> {
        let rsv = self.get_mem_rsv_map()?;
        let mem_area =
            root_node
                .find_nodes("memory")
                .iter()
                .try_fold(vec![], |mut acc, &val| {
                    self.add_mem_to_general(&mut acc, val, &rsv, cell_sz)?;
                    Ok(acc)
                })?;
        Ok(mem_area)
    }

    /// Get the memory reservation map. The reserved memory block are aligned to a page
    ///
    /// **The reserved memory block are not promised to be not overlapped**
    ///
    /// The result should contain the fdt itself.
    fn get_mem_rsv_map(&self) -> Result<Vec<Range<usize>>, Self::TError>;

    /// Initialize basic devices.
    fn init_devices(&self) -> Result<EmbeddedDeviceInfo, Self::TError> {
        let guard = self.get_root_node_lock()?.read();
        let root_node = guard
            .as_ref()
            .ok_or(DeviceTreeError::NotInitializedError)
            .map_err(Self::wrap_error)?;

        let cell_sz = self.get_cell_size()?;
        validate_cell_size(&cell_sz).map_err(Self::wrap_error)?;

        let mem_area = self.calc_mem_area(root_node, &cell_sz)?;

        let info = EmbeddedDeviceInfo {
            general_mem: mem_area,
        };
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
    UnsupportedAddressType {
        address_cells: usize,
        size_cells: usize,
    },
    /// A property had an invalid format when parsed.
    InvalidPropFormat {
        prop_name: MaybeOwnedStr,
        err: InvalidPropFormatError,
    },
    /// Device tree parsing or initialization has not been performed yet.
    NotInitializedError,
}
impl MessageError for DeviceTreeError {}

#[derive(Debug)]
pub enum InvalidPropFormatError {
    /// The parsed array length did not match the expected count.
    ArrayLengthError {
        /// Human readable element type name (e.g. "u32").
        array_type: &'static str,
        /// Actual element count found.
        len: usize,
        /// Expected/factor used when validating length.
        supposed_fact: usize,
    },
}
