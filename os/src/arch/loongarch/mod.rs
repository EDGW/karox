//! Arch-specified functions for Loongarch

use bitflags::bitflags;

pub mod mm;
pub mod reg;
mod sbi;

// TODO:Temporarily Used
#[allow(missing_docs)]
pub type SBITable = sbi::SBITable;

bitflags! {
    /// Priority Levels in Loongarch, presented in bit flags
    pub struct CombinablePriority: u8{
        /// PLV0 Available
        const PLV0 = 0b0001;
        /// PLV1 Available
        const PLV1 = 0b0010;
        /// PLV2 Available
        const PLV2 = 0b0100;
        /// PLV3 Available
        const PLV3 = 0b1000;
    }
}
