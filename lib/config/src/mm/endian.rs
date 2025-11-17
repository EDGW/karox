//! Different architectures and data structures use different byte orders to store data
//! This module is designed to provide convenience to convert data between different endiannesses

/// Represents byte order (endianness) used to interpret multi-byte values.
#[derive(PartialEq, Eq)]
pub enum Endianness {
    /// Most Significant Byte (MSB) comes first in memory (Big-endian).
    Big = 0,
    /// Least Significant Byte (LSB) comes first in memory (Little-endian).
    Little = 1,
}

/// Convert a 16-bit integer between little-endian and big-endian
pub const fn swap_u16(x: u16) -> u16 {
    ((x & 0x00FF) << 8) | ((x & 0xFF00) >> 8)
}

/// Convert a 32-bit integer between little-endian and big-endian
pub const fn swap_u32(x: u32) -> u32 {
    ((x & 0x000000FF) << 24) |
    ((x & 0x0000FF00) << 8)  |
    ((x & 0x00FF0000) >> 8)  |
    ((x & 0xFF000000) >> 24)
}

/// Convert a 64-bit integer between little-endian and big-endian
pub const fn swap_u64(x: u64) -> u64 {
    ((x & 0x00000000000000FF) << 56) |
    ((x & 0x000000000000FF00) << 40) |
    ((x & 0x0000000000FF0000) << 24) |
    ((x & 0x00000000FF000000) << 8)  |
    ((x & 0x000000FF00000000) >> 8)  |
    ((x & 0x0000FF0000000000) >> 24) |
    ((x & 0x00FF000000000000) >> 40) |
    ((x & 0xFF00000000000000) >> 56)
}

/// Get the endianness related to the current architecture
#[macro_export]
#[cfg(target_arch = "riscv64")]
macro_rules! arch_endianness {
    () => {
        Endianness::Little
    };
}

/// Get the endianness related to the current architecture
#[macro_export]
#[cfg(target_arch = "loongarch64")]
macro_rules! arch_endianness {
    () => {
        Endianness::Little
    };
}

/// Swap bytes if source and destination endianness differ
pub fn convert_u16(x: u16, src: Endianness, dst: Endianness) -> u16 {
    if src == dst { x } else { swap_u16(x) }
}

/// Swap bytes if source and destination endianness differ
pub fn convert_u32(x: u32, src: Endianness, dst: Endianness) -> u32 {
    if src == dst { x } else { swap_u32(x) }
}

/// Swap bytes if source and destination endianness differ
pub fn convert_u64(x: u64, src: Endianness, dst: Endianness) -> u64 {
    if src == dst { x } else { swap_u64(x) }
}

/// Convert a value from platform-native to the specified endianness
pub fn to_endian_u16(x: u16, target: Endianness) -> u16 {
    convert_u16(x, arch_endianness!(), target)
}

/// Convert a value from platform-native to the specified endianness
pub fn to_endian_u32(x: u32, target: Endianness) -> u32 {
    convert_u32(x, arch_endianness!(), target)
}

/// Convert a value from platform-native to the specified endianness
pub fn to_endian_u64(x: u64, target: Endianness) -> u64 {
    convert_u64(x, arch_endianness!(), target)
}

/// Convert a value from the specified endianness to platform-native
pub fn from_endian_u16(x: u16, src: Endianness) -> u16 {
    convert_u16(x, src, arch_endianness!())
}

/// Convert a value from the specified endianness to platform-native
pub fn from_endian_u32(x: u32, src: Endianness) -> u32 {
    convert_u32(x, src, arch_endianness!())
}

/// Convert a value from the specified endianness to platform-native
pub fn from_endian_u64(x: u64, src: Endianness) -> u64 {
    convert_u64(x, src, arch_endianness!())
}

/// The packed u32 type represented in big endianness
#[derive(Clone, Copy)]
pub struct BigEndian32(u32);
/// The packed u32 type represented in little endianness
#[derive(Clone, Copy)]
pub struct LittleEndian32(u32);
/// The packed u64 type represented in big endianness
#[derive(Clone, Copy)]
pub struct BigEndian64(u64);
/// The packed u64 type represented in little endianness
#[derive(Clone, Copy)]
pub struct LittleEndian64(u64);

impl BigEndian32{
    /// The the valid value for the current arch
    pub fn value(&self)-> u32 {
        from_endian_u32(self.0, Endianness::Big)
    }
}
impl LittleEndian32{
    /// The the valid value for the current arch
    pub fn value(&self)-> u32 {
        from_endian_u32(self.0, Endianness::Little)
    }
}
impl BigEndian64{
    /// The the valid value for the current arch
    pub fn value(&self)-> u64 {
        from_endian_u64(self.0, Endianness::Big)
    }
}
impl LittleEndian64{
    /// The the valid value for the current arch
    pub fn value(&self)-> u64 {
        from_endian_u64(self.0, Endianness::Little)
    }
}