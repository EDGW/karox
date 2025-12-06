#[cfg(all(target_arch = "riscv64", feature = "naked"))]
pub mod riscv_sbi;

#[cfg(all(target_arch = "loongarch64", feature = "naked"))]
pub mod loongarch_naked;

pub mod shared;