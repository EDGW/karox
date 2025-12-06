mod device_tree;
mod fdt;
pub use device_tree::DeviceTree;
pub use fdt::FdtTree;

#[derive(Debug)]
pub struct MemoryAreaInfo {
    pub start: usize,
    pub length: usize,
}
