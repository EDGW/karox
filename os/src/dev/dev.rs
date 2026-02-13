//! Device management and initialization.
//!
//! Build and initialize a simple device tree composed of [Device] nodes.
//!
//! Overview and architecture:
//! - Represent devices as nodes of a tree using [Device] and [DeviceInfo].
//! - Use `DeviceInfo.comp_list` entries to discover drivers via [find_drivers].
//! - Call driver probe routines to bind drivers to devices. On successful probe,
//!   mark the device and **do not descend into its children** (probe wins over recursion).
//! - If no driver binds, recurse into children and attempt to initialize them.
//!
//! Important notes:
//! - Use [Handle<Device>] to own nodes and [HandleRef<Device>] for weak parent refs.
//! - **When a parent handle exists but is invalid, path construction emits the literal `"[invalid]"`.**
//! - Protect driver status updates via `DeviceInfo.drv_stat` (an RwLock).
//! - Keep doc references to types in square brackets, and prefer imperative sentences.

use crate::{
    debug_ex,
    dev::{
        driver::{Driver, find_drivers},
        handle::{Handle, HandleRef},
        mmio::IoRange,
    },
};
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use bitflags::bitflags;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use log::warn;
use spin::RwLock;

// region: Device

/// [Device] is one node in the global device tree.
#[derive(Debug)]
pub struct Device {
    /// Unique identifier allocated from a global counter.
    pub id: usize,
    /// Weak reference to parent node; `None` for the root.
    pub parent: Option<HandleRef<Device>>,
    /// Owned children handles.
    pub children: RwLock<Vec<Handle<Device>>>,
    /// Metadata and resources for this device.
    pub info: DeviceInfo,
}

/// Metadata and hardware resources for a [Device].=
#[derive(Debug)]
pub struct DeviceInfo {
    pub name: Box<str>,

    /// Compatible identifiers used to find drivers. First match is tried first.
    pub comp_list: Vec<Box<str>>,
    /// Device classification flags.
    pub dev_type: DeviceType,
    /// Driver probe status.
    pub drv_stat: RwLock<DriverStatus>,
    /// Memory-mapped IO ranges assigned to this device.
    pub io_addr: Vec<IoRange>,
    /// Interrupt routing entries for the device.
    pub intr_info: Vec<IntrInfo>,
    /// Optional interrupt-controller info if this device implements an interrupt controller.
    pub intc_info: Option<IntcInfo>,
}

bitflags! {
    /// [DeviceType] flags classify device roles and capabilities.
    ///
    /// Use flags to mark special behavior (for example `INTC` indicates interrupt-controller).
    pub struct DeviceType : u32{
        /// No specific type set.
        const UNSPECIFIED = 0;
        /// Generic device.
        const DEVICE = 0b0001;
        /// Interrupt controller device.
        const INTC = 0b0010;
    }
}

/// [DriverStatus] records probe outcomes for a device.
#[derive(Debug)]
pub enum DriverStatus {
    /// No driver recognized the device yet.
    Unrecognized,
    /// Driver probe failed.
    InitFailed,
    /// Driver probe succeeded and bound.
    Success { driver: &'static dyn Driver },
}

impl DriverStatus {
    pub fn is_success(&self) -> bool {
        match self {
            DriverStatus::Success { driver: _ } => true,
            _ => false,
        }
    }
    pub fn get_driver(&self) -> Option<&'static dyn Driver> {
        match self {
            DriverStatus::Success { driver } => Some(*driver),
            _ => None,
        }
    }
}

/// [IntrInfo] describes routing of one hardware interrupt via an interrupt controller.
#[derive(Debug)]
pub struct IntrInfo {
    /// Interrupt controller instance id.
    pub intc_id: usize,
    /// Interrupt route/index within the controller.
    pub ir: usize,
}

/// [IntcInfo] describes an interrupt-controller implementation on a device.
#[derive(Debug)]
pub struct IntcInfo {
    /// Interrupt-controller instance id.
    pub intc_id: usize,
}

// endregion

// region: DeviceHandle & DeviceRef

static DEV_ID_ALLOC: AtomicUsize = AtomicUsize::new(0);

impl Handle<Device> {
    /// Create a new child [Device] handle with given `info` and set this handle as parent.
    ///
    /// Note: the returned handle is not inserted into `self.children`; call [Handle<Device>::add] to attach it.
    pub fn new_child(&self, info: DeviceInfo) -> Handle<Device> {
        Handle::<Device>::from(Device {
            id: DEV_ID_ALLOC.fetch_add(1, Ordering::SeqCst),
            parent: Some(self.create_ref()),
            children: RwLock::new(vec![]),
            info,
        })
    }

    /// Add this handle to its parent's children list and return a [HandleRef<Device>].
    ///
    /// Return `Err(DeviceHandleError::DeviceNotFound)` if parent or parent handle is missing.
    pub fn add(self) -> Result<HandleRef<Device>, DeviceRegistryError> {
        let res = self.create_ref();
        let handle = self
            .parent
            .as_ref()
            .ok_or(DeviceRegistryError::DeviceNotFound)?
            .get_handle()
            .ok_or(DeviceRegistryError::DeviceNotFound)?;
        handle.children.write().push(self);
        Ok(res)
    }
}

impl Device {
    /// Build a textual path to this device by walking parent references.
    ///
    /// Behavior notes:
    /// - Walk parents recursively and join segments with `/`.
    /// - **If a parent handle is invalid, insert the literal `"[invalid]"` for that segment.**
    /// - Return an owned `String`.
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
        ances + self.info.name.as_ref()
    }

    /// Return the device path as a boxed string.
    ///
    /// Behavior notes:
    /// - Walk parents recursively and join segments with `/`.
    /// - **If a parent handle is invalid, insert the literal `"[invalid]"` for that segment.**
    pub fn get_path(&self) -> Box<str> {
        self.get_path_internal().into_boxed_str()
    }

    /// Create a root [Device] handle (no parent) from `info`.
    fn device_root(info: DeviceInfo) -> Handle<Device> {
        let dev = Device {
            id: DEV_ID_ALLOC.fetch_add(1, Ordering::SeqCst),
            parent: None,
            children: RwLock::new(vec![]),
            info,
        };
        Handle::<Device>::from(dev)
    }
}

#[derive(Debug)]
/// Errors returned by [Handle<Device>] operations.
pub enum DeviceRegistryError {
    /// Parent device not found when attempting to attach a child.
    DeviceNotFound,
}

// endregion

lazy_static! {
    /// Global device root node named "dev".
    ///
    /// The runtime device tree is rooted here. Drivers and platform code should
    /// attach child nodes to this root during early initialization.
    pub static ref DEVICE_ROOT: Handle<Device> = Device::device_root(DeviceInfo {
        name: Box::from("dev"),
        comp_list: vec![],
        dev_type: DeviceType::UNSPECIFIED,
        drv_stat: RwLock::new(DriverStatus::Unrecognized),
        io_addr: vec![],
        intr_info: vec![],
        intc_info: None
    });
}

/// Initialize devices at `current` by attempting to bind drivers.
///
/// Process:
/// 1. For each entry in `current.info.comp_list`, call [find_drivers].
/// 2. If drivers exist, attempt `driver.probe(current.clone())` on the first candidate.
///    - On success: set `drv_stat` to `Success`, **do not recurse into children**, and return Ok.
///    - On failure: set `drv_stat` to `InitFailed` and continue trying other components.
/// 3. If no driver binds, recurse into children via `init_devs_under_node`.
/// 4. Return `Err(())` if any probe or recursive initialization fails.
///
/// Notes:
/// - **Successful probe prevents descending into children**.
/// - Update `drv_stat` under RwLock to reflect probe outcomes.
/// - **Do not roll back on errors.**
pub fn init_devs_by_node(current: &Handle<Device>) -> Result<(), ()> {
    let mut recurse = true;
    let mut failed = false;
    for comp in &current.info.comp_list {
        let drvs = find_drivers(comp);
        if let Some(driver) = drvs.first() {
            match driver.probe(current.clone()) {
                Ok(()) => {
                    *current.info.drv_stat.write() = DriverStatus::Success { driver: *driver };
                    failed = false;
                    recurse = false;
                    debug_ex!(
                        "\tInitialized device '{}' with driver '{}'.",
                        current.get_path(),
                        driver.get_name()
                    );
                    break;
                }
                Err(err) => {
                    *current.info.drv_stat.write() = DriverStatus::InitFailed;
                    failed = true;
                    warn!(
                        "Unable to initialize device '{}' with driver '{}': {:?}.",
                        current.get_path(),
                        driver.get_name(),
                        err
                    );
                }
            }
        }
    }
    if recurse && let Err(()) = init_devs_under_node(current) {
        failed = true;
    }
    if failed { Err(()) } else { Ok(()) }
}

/// Initialize all children of `current`.
///
/// Walk `current.children` and call `init_devs_by_node` for each child.
/// Return `Err(())` if any child fails to initialize.
/// Notes:
/// - **Successful probe prevents descending into children**.
/// - **Do not roll back on errors.**
pub fn init_devs_under_node(current: &Handle<Device>) -> Result<(), ()> {
    let mut failed = false;
    let children = current.children.read();
    for sub in &*children {
        if let Err(()) = init_devs_by_node(sub) {
            failed = true;
        }
    }
    if failed { Err(()) } else { Ok(()) }
}

/// Start device initialization from the global [DEVICE_ROOT].
///
/// Print debug traces and ignore individual errors.
pub fn init_devs() {
    debug_ex!("Initializing devices...");
    let _ = init_devs_by_node(&DEVICE_ROOT);
    debug_ex!("Devices initialized.");
}
