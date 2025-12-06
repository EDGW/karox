use core::ptr::write_volatile;

use crate::arch::symbols::{_ebss, _kbss};

pub fn clear_bss() {
    unsafe {
        let mut p = _kbss as *mut u8;
        let e = _ebss as *mut u8;
        while p < e {
            write_volatile(p, 0);
            p = p.add(1);
        }
    }
}
