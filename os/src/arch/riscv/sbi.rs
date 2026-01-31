use crate::arch::SbiTrait;
use core::arch::asm;

pub struct SbiTable;

pub const SBI_CON_PUTCHR: (usize, usize) = (0x01, 0);
//pub const CON_GETCHR: (usize, usize) = (0x02, 0);
pub const SBI_SET_TIMER: (usize, usize) = (0x54494D45, 0);
pub const SBI_HART_START: (usize, usize) = (0x48534D, 0);

impl SbiTrait for SbiTable {
    fn console_putchr(chr: char) -> Result<(), usize> {
        sbi_call(SBI_CON_PUTCHR, chr as usize, 0, 0)?;
        Ok(())
    }
    fn init() {}
}
impl SbiTable {
    pub fn set_timer(time: usize) -> Result<(), usize> {
        sbi_call(SBI_SET_TIMER, time, 0, 0)?;
        Ok(())
    }
    /// The `a1` register of the given hart will be filled with `opaque`.
    pub fn hart_start(hart_id: usize, start_addr: usize, opaque: usize) -> Result<(), usize> {
        sbi_call(SBI_HART_START, hart_id, start_addr, opaque)?;
        Ok(())
    }
}

#[inline(always)]
fn sbi_call(
    eid_fid: (usize, usize),
    arg0: usize,
    arg1: usize,
    arg2: usize,
) -> Result<usize, usize> {
    let (eid, fid) = eid_fid;
    let mut ret_a0: usize;
    let mut ret_a1: usize;
    unsafe {
        asm!(
            // "li x16, 0",
            "ecall",
            inlateout("a0") arg0 => ret_a0,
            inlateout("a1") arg1 => ret_a1,
            in("a2") arg2,
            in("a6") fid,
            in("a7") eid,
        );
    }
    if ret_a0 != 0 {
        return Err(ret_a0);
    } else {
        return Ok(ret_a1);
    }
}
