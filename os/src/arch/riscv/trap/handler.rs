use core::{arch::naked_asm, fmt::Debug, str};

use riscv::register::sstatus::Sstatus;

use crate::{
    arch::trap::intr::{InterruptTypes, intr_handler},
    kserial_println,
};

pub const EXCEPTION_DESC: [&'static str; 16] = {
    let mut res = ["Reserved or Designated for Custom Use"; 16];
    res[0] = "Instruction Address Misaligned";
    res[1] = "Instruction Access Fault";
    res[2] = "Illegal Instruction";
    res[3] = "Breakpoint";
    res[4] = "Load Address Misaligned";
    res[5] = "Load Access Fault";
    res[6] = "Store/AMO Address Misaligned";
    res[7] = "Store/AMO Access Fault";
    res[8] = "Environment Call from U-mode";
    res[9] = "Environment Call from S-mode";
    res[12] = "Instruction Page Fault";
    res[13] = "Load Page Fault";
    res[15] = "Store/AMO Page Fault";
    res
};

pub fn get_exception_desc(code: usize) -> &'static str {
    if code >= EXCEPTION_DESC.len() {
        "Reserved or Designated for Custom Use"
    } else {
        EXCEPTION_DESC[code]
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _trap_from_kernel() -> ! {
    naked_asm!("
        .altmacro
        .macro SAVE_GP n
            sd x\\n, \\n*8(sp)
        .endm
        .macro LOAD_GP n
            ld x\\n, \\n*8(sp)
        .endm

        .align 2
        // preserve kernel stack

        // add stack frame
        addi   sp, sp, -36*8

        // save registers
        .set n, 1
        .rept 31
            SAVE_GP %n
            .set n, n+1
        .endr

        csrr t0, sstatus
        csrr t1, sepc
        csrr t2, scause
        csrr t3, stval
        sd  t0, 0x100(sp)
        sd  t1, 0x108(sp)
        sd  t2, 0x110(sp)
        sd  t3, 0x118(sp)

        // call
        mv  a0, sp
        call {trap_handler}
        
        // resume registers
        ld t1, 0x108(sp)
        csrw sepc, t1

        .set n, 1
        .rept 31
            LOAD_GP %n
            .set n, n+1
        .endr

        // resume stack
        addi   sp, sp, 36*8
        sret
        
    ",
    trap_handler = sym trap_handler);
}

#[derive(Copy, Clone)]
pub struct TrapContext {
    pub xreg: [usize; 32],
    pub status: Sstatus,
    pub sepc: usize,
    pub scause: usize,
    pub stval: usize,
}
impl Debug for TrapContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Registers:\n"))?;
        for i in 0..32 {
            f.write_fmt(format_args!("x{}:\t{:#x}\n", i, self.xreg[i]))?;
        }
        f.write_fmt(format_args!("sstatus:\t{:?}\n", self.status))?;
        f.write_fmt(format_args!("sepc:\t{:#x}\n", self.sepc))?;
        f.write_fmt(format_args!("scause:\t{:#x}\n", self.scause))?;
        f.write_fmt(format_args!("stval:\t{:#x}\n", self.stval))?;
        Ok(())
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn trap_handler(context: &mut TrapContext) {
    let is_intr = context.scause >> 63 == 1;
    let cause = context.scause & ((1 << 63) - 1);
    if is_intr {
        match InterruptTypes::try_from(cause) {
            Ok(int) => intr_handler(int),
            Err(_) => kserial_println!("Unknown Interrupt Type {:#x}", cause),
        }
    } else {
        kserial_println!(
            "Unexcepted Exception {:#x}({:}) Occurred in kernel at {:#x}",
            cause,
            get_exception_desc(cause),
            context.sepc
        );
        kserial_println!("Trap Info:\n{:?}", context);
        panic!()
    }
}
