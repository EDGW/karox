//! This module defines the offset values for loongarch CSRs

/// Current Mode CSR
pub const CR_CRMD: u16 = 0x0;

/// Previous Mode CSR
pub const CR_PRMD: u16 = 0x1;

/// CPUID CSR
pub const CR_CPUID: u16 = 0x20;

/// Direct Mapped Window 0 CSR
pub const CR_DMW0: u16 = 0x180;

/// Direct Mapped Window 1 CSR
pub const CR_DMW1: u16 = 0x181;

/// Direct Mapped Window 2 CSR
pub const CR_DMW2: u16 = 0x182;
