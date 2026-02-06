use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};
use core::ops::Deref;
use lazy_static::lazy_static;
use spin::RwLock;
use utils::{impl_conversion, impl_deref};

#[derive(Debug)]
pub struct Device {
    pub name: Box<str>,
    pub parent: Option<DeviceRef>,
    pub child: RwLock<Vec<DeviceHandle>>,
}

#[derive(Debug)]
pub struct DeviceHandle {
    inner: Arc<Device>,
}
impl Deref for DeviceHandle {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
impl_conversion!(DeviceHandle, Arc<Device>);

#[derive(Debug)]
pub struct DeviceRef {
    inner: Weak<Device>,
}

impl Clone for DeviceRef {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl_deref!(DeviceRef, Weak<Device>);
impl_conversion!(DeviceRef, Weak<Device>);

impl DeviceHandle {
    fn device_root(name: impl AsRef<str>) -> DeviceHandle {
        DeviceHandle::from(Arc::new(Device {
            name: Box::from(name.as_ref()),
            parent: None,
            child: RwLock::new(vec![]),
        }))
    }

    pub fn create_ref(&self) -> DeviceRef {
        DeviceRef::from(Arc::downgrade(&self.inner))
    }

    pub fn new_child(&self, name: impl AsRef<str>) -> DeviceHandle {
        DeviceHandle::from(Arc::new(Device {
            name: Box::from(name.as_ref()),
            parent: Some(self.create_ref()),
            child: RwLock::new(vec![]),
        }))
    }

    pub fn add(self) -> Result<DeviceRef, DeviceHandleError> {
        let res = self.create_ref();
        let handle = self
            .parent
            .as_ref()
            .ok_or(DeviceHandleError::DeviceNotFound)?
            .get_handle()
            .ok_or(DeviceHandleError::DeviceNotFound)?;
        handle.child.write().push(self);
        Ok(res)
    }
}

#[derive(Debug)]
pub enum DeviceHandleError {
    DeviceNotFound,
}

impl DeviceRef {
    pub fn get_handle(&self) -> Option<DeviceHandle> {
        Some(DeviceHandle::from(Weak::upgrade(&self.inner)?))
    }
}

lazy_static! {
    pub static ref DEVICE_ROOT: DeviceHandle = DeviceHandle::device_root("dev");
}
