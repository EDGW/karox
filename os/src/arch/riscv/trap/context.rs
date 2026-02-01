use crate::task::hart::{HART_INFO, HartInfo};
use core::fmt::Debug;
use riscv::register::sstatus::{self, SPP, Sstatus};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    hart_info_ptr: usize,
    kstack_top: usize,
}

impl TrapContext {
    pub fn zero_from_entry(
        entry: *const (),
        hart_id: usize,
        kernel_mode: bool,
        tstack_top: usize,
        kstack_top: usize,
        tp: usize,
    ) -> TrapContext {
        let hart_info = &HART_INFO[hart_id];
        let mut status = sstatus::read();
        if kernel_mode {
            status.set_spp(SPP::Supervisor);
        } else {
            status.set_spp(SPP::User);
        }
        status.set_spie(true);
        TrapContext {
            x: {
                let mut x = [0; 32];
                x[2] = tstack_top; // sp
                x[4] = tp; //tp
                x
            },
            sstatus: status,
            sepc: entry as usize,
            hart_info_ptr: hart_info as *const HartInfo as usize,
            kstack_top: kstack_top,
        }
    }
}

impl Debug for TrapContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrapContext")
            .field("x", &self.x)
            .field("sstatus", &format_args!("{:#x}", self.sstatus.bits()))
            .field("sepc", &format_args!("{:#x}", self.sepc))
            .field("hart_info_ptr", &format_args!("{:#x}", self.hart_info_ptr))
            .field("kstack_top", &format_args!("{:#x}", self.kstack_top))
            .finish()
    }
}
