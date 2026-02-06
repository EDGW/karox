use crate::dev::mmio::Register;

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

    /// Context Registers[0x200000,0x400000]
    contexts: [ContextRegisters; 0x3e00],
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
}

impl PLIntrController {
    pub fn claim(&self, cxt_id: usize) -> u32 {
        self.registers.contexts[cxt_id].claim_comp.read()
    }

    pub fn complete(&self, cxt_id: usize, irq_id: u32) {
        self.registers.contexts[cxt_id].claim_comp.write(irq_id);
    }
}
