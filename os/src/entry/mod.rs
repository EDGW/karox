//! ## Entry Module
//! The entry is where the operating system starts from
//!
//! The entry performs pre-initializations according to its architecture and boot-modes,
//!     defined through the `target_arch` and `feature`s config.
//!
//! After the pre-initialization:
//! - the high half kernel address space should be properly initialized (e.g. through setting up the early boot page table);
//! - a device info struct (uninitialized) should be created;
//! - the bss section of the kernel should be cleared,
//! and then the entry should call [crate::rust_main].
//!
//! Since different architectures may use different types to describe the device info,
//!     the entry should choose a proper implementation of [crate::devices::device_info::DeviceInfo] and send it to [crate::rust_main]
//!
//! All the members in this module is **private**, because the entry should **never** be used after the pre-initialization.
//!

#[cfg(all(target_arch = "riscv64", feature = "naked"))]
mod riscv_sbi;

#[cfg(all(target_arch = "loongarch64", feature = "naked"))]
mod loongarch_naked;

mod shared;
