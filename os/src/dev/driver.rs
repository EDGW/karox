use alloc::{boxed::Box, collections::btree_map::BTreeMap, vec, vec::Vec};
use spin::RwLock;
use utils::vec::LockedVecStatic;

use crate::{
    debug_ex,
    dev::{DeviceHandle, ic},
};
use core::fmt::Debug;

pub trait Driver: Sync {
    fn get_name(&self) -> &'static str;
    fn get_comp_strs(&self) -> &'static [&'static str];
    fn probe(&self, dev: DeviceHandle) -> Result<(), Box<dyn Debug>>;
}

static COMP_MAP: RwLock<BTreeMap<&'static str, Vec<&'static dyn Driver>>> =
    RwLock::new(BTreeMap::new());
static DRIVERS: LockedVecStatic<dyn Driver> = LockedVecStatic::new();

pub fn find_drivers(comp_str: &str) -> Vec<&'static dyn Driver> {
    let guard = COMP_MAP.read();
    if let Some(drv) = guard.get(comp_str) {
        drv.clone()
    } else {
        vec![]
    }
}

pub fn register_driver<T: 'static + Driver>(driver: Box<T>) {
    debug_ex!("\tRegistered driver '{}'.", driver.get_name());
    let (driver, _) = DRIVERS.push_boxed(driver);
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

pub fn init() {
    debug_ex!("Registering drivers...");
    ic::register_drivers();
    debug_ex!("Drivers registered.");
}
