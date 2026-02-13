use core::fmt::Debug;

use crate::{
    arch::hart::get_current_hart_id,
    dev::{
        Device,
        driver::{Driver, DriverProbeError, IntcError, MmioError},
        handle::Handle,
        intc::{Intc, IntcDev, register_intc},
        mmio::{IoRangeValidationType, reg::Register},
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use spin::Mutex;

/// Platform-Level Interrupt Controller Register Map
#[repr(C)]
struct PLICRegisters {
    /// Priorities[0x0,0x1000). `prio[0]` is reserved
    prio: [Register<u32>; 0x400],

    /// Pending Bits[0x1000,0x1080)
    pending: [Register<u32>; 0x20],

    /// Reserved[0x1080,0x2000)
    _rsv1: [Register<u32>; 0x3e0],

    /// Enable Bits[0x2000,0x1f2000)
    enable: [[Register<u32>; 0x20]; 0x3e00],

    /// Reserved[0x1f2000,0x200000)
    _rsv2: [Register<u32>; 0x3800],

    /// Context Registers[0x200000,0x4000000], at least [0x200000, 0x600000]
    contexts: [ContextRegisters; 1024],
}

#[repr(C)]
struct ContextRegisters {
    /// Priority Threshold[0x0,0x4)
    prio_thres: Register<u32>,

    /// Claim or Complete[0x4,0x8)
    claim_comp: Register<u32>,

    /// Reserved[0x8,0x1000)
    _rsv1: [Register<u32>; 0x3fe],
}

pub struct PLIntrController {
    registers: &'static PLICRegisters,
    locker: Mutex<()>,
}

impl Debug for PLIntrController {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PLIntrController").finish()
    }
}

unsafe impl Sync for PLIntrController {}
unsafe impl Send for PLIntrController {}

impl PLIntrController {
    pub fn new(reg_addr: usize) -> Result<PLIntrController, MmioError> {
        let registers = unsafe { (reg_addr as *const u8 as *const PLICRegisters).as_ref() }
            .ok_or(MmioError::InvalidAddress)?;
        Ok(PLIntrController {
            registers,
            locker: Mutex::new(()),
        })
    }

    #[inline(always)]
    pub fn get_context_id(&self) -> usize {
        get_current_hart_id() * 2 + 1
    }
}

impl IntcDev for PLIntrController {
    fn claim(&self) -> usize {
        let ctx_id = self.get_context_id();
        let guard = self.locker.lock();
        let res = self.registers.contexts[ctx_id].claim_comp.read();
        drop(guard);
        res as usize
    }

    fn complete(&self, irq_id: usize) {
        let ctx_id = self.get_context_id();
        let guard = self.locker.lock();
        self.registers.contexts[ctx_id]
            .claim_comp
            .write(irq_id as u32);
        drop(guard);
    }
}

#[derive(Debug)]
pub struct PLICDriver;
impl PLICDriver {
    pub fn new() -> PLICDriver {
        PLICDriver
    }
}

static PLIC_DEVS: Mutex<Vec<Handle<Intc>>> = Mutex::new(vec![]);

impl Driver for PLICDriver {
    fn get_name(&self) -> &'static str {
        "SiFive RISC-V PLIC"
    }

    fn get_comp_strs(&self) -> &'static [&'static str] {
        &["sifive,plic-1.0.0"]
    }

    fn probe(&self, dev: Handle<Device>) -> Result<(), DriverProbeError> {
        let io_addr = &dev.info.io_addr;
        if io_addr.is_empty() {
            return Err(DriverProbeError::Mmio(MmioError::AddressNotSpecified));
        }
        let io_addr = &io_addr[0];
        if !io_addr.validate::<PLICRegisters>(IoRangeValidationType::Compatible) {
            return Err(DriverProbeError::Mmio(MmioError::NotEnoughSpace));
        }
        let intc_id = dev
            .info
            .intc_info
            .as_ref()
            .ok_or(DriverProbeError::Intc(IntcError::IdNotGiven))?
            .intc_id;
        let io_addr = io_addr.start;
        let dev = PLIntrController::new(io_addr).map_err(|err| DriverProbeError::Mmio(err))?;
        let handle =
            register_intc(intc_id, Box::new(dev)).map_err(|err| DriverProbeError::Intc(err))?;
        PLIC_DEVS.lock().push(handle);
        Ok(())
    }

    fn on_registered(&self) {}
}
