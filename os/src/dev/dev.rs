use alloc::{
    boxed::Box,
    string::String,
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};
use core::ops::Deref;
use lazy_static::lazy_static;
use log::warn;
use spin::RwLock;
use utils::{impl_conversion, impl_deref};

use crate::{debug_ex, dev::driver::find_drivers, panic_init};

#[derive(Debug)]
pub struct Device {
    pub name: Box<str>,
    pub comp_list: Vec<Box<str>>,
    pub parent: Option<DeviceRef>,
    pub children: RwLock<Vec<DeviceHandle>>,
    pub dev_type: DeviceType,
    pub drv_stat: RwLock<DriverStatus>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DriverStatus {
    Unrecognized,
    InitFailed,
    Success,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeviceType {
    IntrController,
    Device,
    Unspecified,
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
    fn get_path_internal(&self) -> String {
        let ances = match &self.parent {
            Some(parent) => {
                if let Some(handle) = parent.get_handle() {
                    handle.get_path_internal() + "/"
                } else {
                    String::from("[invalid]/")
                }
            }
            None => String::from("/"),
        };
        ances + self.name.as_ref()
    }
    pub fn get_path(&self) -> Box<str> {
        self.get_path_internal().into_boxed_str()
    }

    fn device_root(
        name: impl AsRef<str>,
        comp_list: Vec<Box<str>>,
        dev_type: DeviceType,
    ) -> DeviceHandle {
        DeviceHandle::from(Arc::new(Device {
            name: Box::from(name.as_ref()),
            parent: None,
            children: RwLock::new(vec![]),
            comp_list,
            dev_type,
            drv_stat: RwLock::new(DriverStatus::Unrecognized),
        }))
    }

    pub fn create_clone(&self) -> DeviceHandle {
        DeviceHandle {
            inner: self.inner.clone(),
        }
    }

    pub fn create_ref(&self) -> DeviceRef {
        DeviceRef::from(Arc::downgrade(&self.inner))
    }

    pub fn new_child(
        &self,
        name: impl AsRef<str>,
        comp_list: Vec<Box<str>>,
        dev_type: DeviceType,
    ) -> DeviceHandle {
        DeviceHandle::from(Arc::new(Device {
            name: Box::from(name.as_ref()),
            parent: Some(self.create_ref()),
            children: RwLock::new(vec![]),
            comp_list,
            dev_type,
            drv_stat: RwLock::new(DriverStatus::Unrecognized),
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
        handle.children.write().push(self);
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
    pub static ref DEVICE_ROOT: DeviceHandle =
        DeviceHandle::device_root("dev", vec![], DeviceType::Unspecified);
}

fn init_devs_by_node(current: &DeviceHandle) {
    let mut recurse = true;
    for comp in &current.comp_list {
        let drvs = find_drivers(comp);
        if let Some(driver) = drvs.first() {
            match driver.probe(current.create_clone()) {
                Ok(()) => {
                    debug_ex!(
                        "\nInitialized device '{}' with driver '{}'",
                        current.get_path(),
                        driver.get_name()
                    );
                    *current.drv_stat.write() = DriverStatus::Success;
                    recurse = false;
                    break;
                }
                Err(err) => {
                    *current.drv_stat.write() = DriverStatus::InitFailed;
                    warn!(
                        "Unable to initialize device '{}' with driver '{}': {:?}.",
                        current.get_path(),
                        driver.get_name(),
                        err.as_ref()
                    );
                }
            }
        }
    }
    if recurse {
        let children = current.children.read();
        for sub in &*children {
            init_devs_by_node(sub);
        }
    }
}

pub fn init() {
    debug_ex!("Initializing devices...");
    init_devs_by_node(&DEVICE_ROOT);
    debug_ex!("Devices initialized...");
}
