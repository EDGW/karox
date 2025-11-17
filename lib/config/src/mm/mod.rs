//! Some memory-management-related configurations,
//! including symbols defined in linker.ld and some arch-specific configurations

pub mod endian;

/// Include a specific symbol in linker.js
#[macro_export]
macro_rules! include_symbol {
    ($name:ident) => {
        unsafe extern "C"{
            /// The pointer to the symbol with the same name defined in linker.js
            pub unsafe fn $name();
        }
    };
}

#[allow(unused_macros)]
/// Convert a symbol(usually defined as functions) to a mutable usize pointer
macro_rules! as_pointer {
    ($name:ident) => {
        ($name as (mut* usize))
    };
}

include_symbol!(_skernel);
include_symbol!(_stext);
include_symbol!(_etext);
include_symbol!(_srodata);
include_symbol!(_erodata);
include_symbol!(_sdata);
include_symbol!(_edata);
include_symbol!(_sbss);
include_symbol!(_kbss);
include_symbol!(_ebss);
include_symbol!(_ekernel);

/// The size of the kernel stack
pub const KERNEL_STACK_SIZE: usize    =   1*1024*1024;

include_arch_files!();