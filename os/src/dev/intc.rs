use core::fmt::Debug;

use crate::{
    debug_ex,
    dev::{
        Device,
        driver::IntcError,
        handle::{Handle, HandleRef},
    },
};
use alloc::{boxed::Box, collections::btree_map::BTreeMap};
use spin::RwLock;

pub trait IntcDev: Sync + Debug {
    fn claim(&self) -> usize;
    fn complete(&self, ir: usize);
}

#[derive(Debug)]
pub struct Intc {
    ctl: Box<dyn IntcDev + Send>,
    pub devs: RwLock<BTreeMap<usize, HandleRef<Device>>>,
}

impl Intc {
    /// None meaning that the intc_id is occupied
    fn new(intc_id: usize, ctl: Box<dyn IntcDev + Send>) -> Option<Handle<Intc>> {
        let mut guard = INTC_MAP.write();
        if guard.contains_key(&intc_id) {
            return None;
        }
        let intc = Intc {
            ctl,
            devs: RwLock::new(BTreeMap::new()),
        };
        let handle = Handle::from(intc);
        guard.insert(intc_id, handle.create_ref());
        Some(handle)
    }
}

impl Drop for Intc {
    fn drop(&mut self) {
        debug_ex!("Intc {:?} dropped", self);
        todo!()
    }
}

static INTC_MAP: RwLock<BTreeMap<usize, HandleRef<Intc>>> = RwLock::new(BTreeMap::new());

pub fn get_intc(id: usize) -> Option<HandleRef<Intc>> {
    INTC_MAP.read().get(&id).cloned()
}

pub fn register_intc(id: usize, ctl: Box<dyn IntcDev + Send>) -> Result<Handle<Intc>, IntcError> {
    let res = Intc::new(id, ctl).ok_or(IntcError::DuplicatedId);
    debug_ex!("Registered interrupt controller #{}.", id);
    res
}
