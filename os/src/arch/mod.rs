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

/// Declare a private module named `arch_name` if the `target_arch` is `arch_str`, and automatically use all the members within.
///
/// Use this macro to prevent from using hard-coded module names to referencing an arch
macro_rules! define_arch {
    ($arch_name:ident, $arch_str:literal) => {
        #[cfg(target_arch = $arch_str)]
        mod $arch_name;
        #[cfg(target_arch = $arch_str)]
        pub use $arch_name::*;
    };
}

define_arch!(riscv, "riscv64");
define_arch!(loongarch, "loongarch64");

pub mod endian;
pub mod symbols;
