//! Driver subsystem: registration, discovery and probe orchestration.
//!
//! Responsibilities:
//! - Provide the [Driver] trait for platform drivers and a global registry that maps
//!   compatible strings to driver implementations.
//! - Keep driver instances alive for the lifetime of the kernel so that `&'static` references
//!   to drivers can be stored and used during probe. The storage used for this purpose is
//!   [DRIVER_REG].
//! - Allow concurrent lookups via [find_drivers] and synchronized updates via [register_driver].
//!
//! Ownership and concurrency notes:
//! - [COMP_MAP] is protected by an [RwLock] for concurrent reader-heavy access patterns.
//! - [DRIVER_REG] owns boxed driver instances and yields `&'static dyn Driver` references.
//! - **Drivers returned by [find_drivers] are `&'static` references originating from [DRIVER_REG].**
use crate::{
    debug_ex,
    dev::{Device, handle::Handle, serial},
};
use alloc::{boxed::Box, collections::btree_map::BTreeMap, vec, vec::Vec};
use core::fmt::Debug;
use spin::RwLock;
use utils::vec::LockedVecStatic;

/// Trait implemented by drivers.
///
/// Responsibilities:
/// - Identify compatible strings via [get_comp_strs] so the global registry can discover candidates.
/// - Implement [probe] to attempt binding to a [Device]. Return `Ok(())` on success or a [DriverProbeError].
/// - Provide an optional hook [on_registered] that runs once at registration time (useful for static init).
///
/// Guarantees and expectations:
/// - Implementations must be `Sync` and have `'static` lifetime when registered.
/// - [probe] may be called multiple times by the initialization logic; a successful probe usually
///   prevents descending into the device's children (see device initialization policy).
pub trait Driver: Sync + Debug {
    fn get_name(&self) -> &'static str;
    fn get_comp_strs(&self) -> &'static [&'static str];
    fn probe(&self, dev: Handle<Device>) -> Result<(), DriverProbeError>;
    fn on_registered(&self);
}

/// Registry mapping from compatible string to candidate driver references.
static COMP_MAP: RwLock<BTreeMap<&'static str, Vec<&'static dyn Driver>>> =
    RwLock::new(BTreeMap::new());

/// Global storage that owns driver instances.
static DRIVER_REG: LockedVecStatic<dyn Driver> = LockedVecStatic::new();

/// Look up drivers matching `comp_str`.
///
/// Return a vector of `&'static dyn Driver` candidate references. The caller receives owned clones
/// of the internal vector to avoid holding locks while probing. If no drivers match, return an empty vec.
pub fn find_drivers(comp_str: &str) -> Vec<&'static dyn Driver> {
    let guard = COMP_MAP.read();
    if let Some(drv) = guard.get(comp_str) {
        drv.clone()
    } else {
        vec![]
    }
}

/// Register a driver instance.
///
/// Steps:
/// 1. Log registration and invoke [Driver::on_registered] for immediate initialization hooks.
/// 2. Push the boxed instance into [DRIVER_REG] to obtain a stable `&'static` reference.
/// 3. Insert the stable reference into [COMP_MAP] under each compatible string returned by the driver.
///
/// Notes:
/// - [on_registered] is called before the instance is placed into [DRIVER_REG]; avoid relying on
///   `&'static` references inside that hook unless the implementation ensures them independently.
/// - Registration is intended to run at boot; do not unregister drivers at runtime.
pub fn register_driver<T: 'static + Driver>(driver: Box<T>) {
    debug_ex!("\tRegistered driver '{}'.", driver.get_name());
    driver.on_registered();
    let (driver, _) = DRIVER_REG.push_boxed(driver);

    let mut guard = COMP_MAP.write();
    for comp in driver.get_comp_strs() {
        let key = *comp;
        if !guard.contains_key(key) {
            guard.insert(key, vec![]);
        }
        let vec = guard.get_mut(key).unwrap();
        vec.push(driver);
    }
}

/// Initialize driver registry by calling platform-specific registrations.
pub fn init() {
    debug_ex!("Registering drivers...");
    {
        use crate::dev::arch::register_drivers;
        register_drivers();
    }
    serial::register_drivers();
    debug_ex!("Drivers registered.");
}

// region: Error Types

/// Errors that may be returned by [Driver::probe].
///
/// Use these variants to express common probe failure reasons; platform code may wrap or
/// convert them into higher-level diagnostics.
#[derive(Debug, Clone, Copy)]
pub enum DriverProbeError {
    /// MMIO-related failures (address missing or invalid).
    Mmio(MmioError),
    /// Interrupt-controller related failures.
    Intc(IntcError),
    /// A sub-device failed to initialize; propagate up.
    SubDeviceError,
    /// Custom driver-specific information.
    Customized { info: &'static str },
}

/// MMIO-related probe failures.
#[derive(Debug, Clone, Copy)]
pub enum MmioError {
    /// MMIO space is not enough.
    NotEnoughSpace,
    /// MMIO address is invalid or out of supported range.
    InvalidAddress,
    /// Device did not specify MMIO resources.
    AddressNotSpecified,
}

/// Interrupt-controller related probe failures.
#[derive(Debug, Clone, Copy)]
pub enum IntcError {
    /// Required interrupt-controller id not provided.
    IdNotGiven,
    /// Controller id already registered or duplicated.
    DuplicatedId,
}

// endregion
