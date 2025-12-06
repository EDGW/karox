//! ## Arch module
//! This module contains some arch-specific contents.
//! 
//! **This module should be kept as minimal as possible**:
//! a function should **only** be implemented here if it is inseparable from the specific architecture.
//! 
//! ### Structure
//! Architecture-specific implementations reside in corresponding subdirectories,
//! declared here and used automatically.
//!
//! This allows you to use `arch::submodules` (referencing `arch::[arch_name]::submodules`)
//! as if the intermediate layer didn't exist.

macro_rules! define_arch {
    ($arch_name:ident, $arch_str:literal) => {
        #[cfg(target_arch = $arch_str)]
        mod $arch_name;
        #[cfg(target_arch = $arch_str)]
        pub use $arch_name::*;
    };
}

define_arch!(riscv,"riscv64");
define_arch!(loongarch,"loongarch64");

mod device_info;
mod sbi;
pub use device_info::*;
pub use sbi::*;
pub mod endian;
pub mod symbols;