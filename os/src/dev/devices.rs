use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};
use lazy_static::lazy_static;
use spin::RwLock;
use utils::{impl_conversion, impl_deref, vec::LockedVecStatic};

#[derive(Debug)]
pub struct Device {
    pub name: Box<str>,
    pub parent: Option<DeviceRef>,
    pub child_fixed: LockedVecStatic<DevicePtr>,
    pub child_hot: RwLock<Vec<DevicePtr>>,
}

#[derive(Debug)]
pub struct DevicePtr {
    inner: Arc<Device>,
}
impl_deref!(DevicePtr, Arc<Device>);
impl_conversion!(DevicePtr, Arc<Device>);

#[derive(Debug)]
pub struct DeviceRef {
    inner: Weak<Device>,
}
impl_deref!(DeviceRef, Weak<Device>);
impl_conversion!(DeviceRef, Weak<Device>);

impl DevicePtr {
    pub fn new_root(name: &str) -> DevicePtr {
        DevicePtr::from(Arc::new(Device {
            name: Box::from(name),
            parent: None,
            child_fixed: LockedVecStatic::new(),
            child_hot: RwLock::new(vec![]),
        }))
    }

    pub fn create_ref(&self) -> DeviceRef {
        DeviceRef::from(Arc::downgrade(self))
    }

    pub fn new_child(&self, name: &str) -> DeviceRef {
        let dev = DevicePtr::from(Arc::new(Device {
            name: Box::from(name),
            parent: Some(self.create_ref()),
            child_fixed: LockedVecStatic::new(),
            child_hot: RwLock::new(vec![]),
        }));
        let res = dev.create_ref();
        self.child_fixed.push(dev);
        res
    }
}

lazy_static! {
    pub static ref DEVICE_ROOT: DevicePtr = DevicePtr::new_root("dev");
}
