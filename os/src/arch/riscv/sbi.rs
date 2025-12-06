use core::arch::{asm};

use crate::arch::SBITrait;

pub struct SBITable;

pub const EID_CON_PUTCHR: (usize,usize)= (0x01,0);
pub const EID_CON_GETCHR: (usize,usize)= (0x02,0);

impl SBITrait for SBITable {
    fn console_putstr(str: &str) -> Result<(),usize> {
        for c in str.as_bytes(){
            let res = sbi_call(EID_CON_PUTCHR, *c as usize, 0, 0);
            if res != 0{
                return Err(res);
            }
        }
        Ok(())
    }
}

#[inline(always)]
fn sbi_call(eid_fid: (usize, usize), arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            // "li x16, 0",
            "ecall",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
            in("a6") eid_fid.1,
            in("a7") eid_fid.0,
        );
    }
    ret
}