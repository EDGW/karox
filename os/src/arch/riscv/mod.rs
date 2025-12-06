//! Arch-specified functions for RISC-V

pub mod mm;
mod sbi;

pub type SBITable = sbi::SBITable;