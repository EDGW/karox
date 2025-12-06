use bitflags::bitflags;

pub mod mm;
mod sbi;
pub mod reg;

pub type SBITable = sbi::SBITable;

bitflags! {
    pub struct PrvLevelBits: u8{
        const PLV0 = 0b0001;
        const PLV1 = 0b0010;
        const PLV2 = 0b0100;
        const PLV3 = 0b1000;
    }
}