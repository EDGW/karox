use core::arch::asm;

use crate::arch::SBITrait;

pub struct SBITable;

pub const CON_PUTCHR: (usize, usize) = (0x01, 0);
//pub const CON_GETCHR: (usize, usize) = (0x02, 0);
pub const SBI_SET_TIMER: (usize, usize) = (0x54494D45, 0);

impl SBITrait for SBITable {
    fn console_putstr(str: &str) -> Result<(), usize> {
        for c in str.as_bytes() {
            let res = sbi_call(CON_PUTCHR, *c as usize, 0, 0);
            if res != 0 {
                return Err(res);
            }
        }
        Ok(())
    }
    fn init() {}
}
impl SBITable {
    pub fn set_timer(time: usize) -> Result<(), usize> {
        let res = sbi_call(SBI_SET_TIMER, time, 0, 0);
        if res == 0 { Ok(()) } else { Err(res) }
    }
}

#[inline(always)]
fn sbi_call(eid_fid: (usize, usize), arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut return_value;
    unsafe {
        asm!(
            // "li x16, 0",
            "ecall",
            inlateout("a0") arg0 => return_value,
            in("a1") arg1,
            in("a2") arg2,
            in("a6") eid_fid.1,
            in("a7") eid_fid.0,
        );
    }
    return_value
}
