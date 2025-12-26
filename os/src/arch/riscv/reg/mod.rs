//! RISC-V Control Registers
//!
//! This module provides definitions and utilities for working with RISC-V control registers

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
                use core::arch::asm;
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
                use core::arch::asm;
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

mod mm;
mod trap;
pub use {mm::*, trap::*};
