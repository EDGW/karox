//! Arch-specified functions for RISC-V

pub mod mm;
mod sbi;

// TODO:Temporarily Used
#[allow(missing_docs)]
pub type SBITable = sbi::SBITable;
