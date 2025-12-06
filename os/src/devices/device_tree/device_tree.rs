use alloc::{
    boxed::Box,
    slice,
    vec,
    vec::Vec,
};
use spin::rwlock::RwLock;

use crate::{
    arch::endian::EndianData,
    devices::device_tree::MemoryAreaInfo,
    error::MessageError,
    mm::types::{MaybeOwned, MaybeOwnedBytes, MaybeOwnedStr},
};


#[repr(C)]
#[derive(Clone, Copy)]
pub struct DPRange<T> {
    pub start: T,
    pub length: T,
}

// endregion
#[derive(Debug, Clone, Copy)]
pub struct RegCellWidth {
    address: u32,
    size: u32,
}

pub struct DeviceNode {
    pub node_name: MaybeOwnedStr,
    pub unit_addr: MaybeOwnedStr,
    pub subnodes: Vec<Box<DeviceNode>>,
    pub props: Vec<DeviceProp>,
}

impl DeviceNode {
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

    pub fn find_prop(&self, name: &str) -> Option<&DeviceProp> {
        self.find_props([name])[0]
    }

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
    pub prop_name: MaybeOwnedStr,
    pub raw_data: MaybeOwnedBytes,
}

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
    pub mem_area: Vec<MemoryAreaInfo>,
}

pub trait DeviceTree {
    type TError: MessageError;
    type TDataType: EndianData<u32>;
    fn init(&self) -> Result<(), Self::TError>;

    fn get_root_node_lock(&self) -> &RwLock<Option<DeviceNode>>;

    fn get_mem_info(&self) -> Result<&Vec<MemoryAreaInfo>, Self::TError>;

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

    /// Init all basic devices
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
    PropertyNotFound {
        prop_name: MaybeOwnedStr,
    },
    NodeNotFound {
        node_name: MaybeOwnedStr,
    },
    UnsupportedAddressType {
        address_cells: u32,
        size_cells: u32,
    },
    InvalidPropFormat {
        prop_name: MaybeOwnedStr,
        err: InvalidPropFormatError,
    },
}

#[derive(Debug)]
pub enum InvalidPropFormatError {
    ArrayLengthError {
        array_type: &'static str,
        len: u32,
        supposed_fact: u32,
    },
}

impl MessageError for DeviceTreeError {}
