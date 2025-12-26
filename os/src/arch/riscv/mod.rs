//! Arch-specified functions for RISC-V

pub mod mm;
pub mod reg;
mod sbi;
pub mod trap;

// TODO:Temporarily Used
#[allow(missing_docs)]
pub type SBITable = sbi::SBITable;
