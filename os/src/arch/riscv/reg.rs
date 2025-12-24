//! RISC-V Control Registers
//! 
//! This module provides definitions and utilities for working with RISC-V control registers

use core::arch::asm;

use bitflags::bitflags;

use crate::{arch::mm::paging::PageNum, define_struct_num};

/// Macro to define a control register.
/// 
/// * `$value` - The type of the value stored in the register.
/// * `$ops_after_set` - Additional operations to perform after setting the value.
macro_rules! define_cr_register {
    ($name: ident, $reg_name: literal, $value: ident, $ops_after_set: literal) => {
        pub struct $name;
        impl $name{
            #[inline(always)]
            pub fn set_value(value: $value){
                unsafe{
                    asm!(
                        concat!("csrw ",$reg_name,",{0}"),
                        $ops_after_set,
                        in(reg) value.get_value(),
                        options(nostack, preserves_flags)
                    );
                }
            }

            #[inline(always)]
            pub fn get_value() -> $value{
                unsafe{
                    let res: usize;
                    asm!(
                        concat!("csrr ","{0},",$reg_name),
                        out(reg) res,
                        options(nostack, preserves_flags)
                    );
                    $value::from_value(res)
                }
            }
        }
    };
}

bitflags! {
    /// Paging modes used in the `satp` register.
    pub struct CrSatpModes : u8{
        /// Non-paging mode.
        const BARE  = 0;
        /// SV39 paging strategy.
        const SV39  = 8;
    }
}

// Represents a packed `satp` register value.
// 
// The `satp` register layout:
// 63   60 59    44 43                     0
// | MODE |  ASID  |         PPN           |
define_struct_num!(CrSatpValue, usize);
impl CrSatpValue {
    /// Creates a new `satp` register value.
    /// 
    /// # Arguments
    /// 
    /// * `mode` - The paging mode.
    /// * `asid` - The address space identifier.
    /// * `ppn` - The physical page number.
    pub const fn create(mode: CrSatpModes, asid: u16, ppn: PageNum) -> CrSatpValue {
        CrSatpValue(((mode.bits as usize) << 60) | ((asid as usize) << 44) | ppn.0)
    }
}

// Defines the `satp` control register.
define_cr_register!(CrSatp, "satp", CrSatpValue, "sfence.vma");
