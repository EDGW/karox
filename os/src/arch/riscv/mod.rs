//! Arch-specified functions for RISC-V

mod config;
pub use config::*;

pub mod hart;
pub mod mm;
mod sbi;
pub mod task;
pub mod trap;

// TODO:Temporarily Used
#[allow(missing_docs)]
pub type SBITable = sbi::SBITable;
