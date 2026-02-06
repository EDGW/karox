//! ## Endianness Module
//! This module provides some structs to better resolve the data in specific endianness rules
//!
//! All the types declared here implements [EndianData<T>],
//! which defines [EndianData<T>::value] function to parse the data into the endianness of the current arch

///[u8] in Big Endianness
#[derive(Debug, Clone, Copy)]
pub struct BigEndian8(u8);

///[u8] in Little Endianness
#[derive(Debug, Clone, Copy)]
pub struct LittleEndian8(u8);

///[u16] in Big Endianness
#[derive(Debug, Clone, Copy)]
pub struct BigEndian16(u16);

///[u16] in Little Endianness
#[derive(Debug, Clone, Copy)]
pub struct LittleEndian16(u16);

///[u32] in Big Endianness
#[derive(Debug, Clone, Copy)]
pub struct BigEndian32(u32);

///[u32] in Little Endianness
#[derive(Debug, Clone, Copy)]
pub struct LittleEndian32(u32);

///[u64] in Big Endianness
#[derive(Debug, Clone, Copy)]
pub struct BigEndian64(u64);

///[u64] in Little Endianness
#[derive(Debug, Clone, Copy)]
pub struct LittleEndian64(u64);

/// This trait defines a packed data in memory with some specific endianness.
pub trait EndianData<T>: Copy + Clone {
    /// Parse the value into the endianness of the current architecture.
    fn value(&self) -> T;
}

/// Get whether the current architecture is big endian
#[cfg(any(
    target_arch = "riscv64",
    target_arch = "loongarch64",
    target_arch = "x86_64"
))]
macro_rules! arch_is_big_endian {
    () => {
        false
    };
}
#[cfg(not(any(
    target_arch = "loongarch64",
    target_arch = "riscv64",
    target_arch = "x86_64"
)))]
macro_rules! arch_is_big_endian {
    () => {
        compile_error!("Unsupported architecture!");
    };
}

/// Implement an [EndianData<T>] for a specific type, and explain the data in big endianess
macro_rules! impl_converter_big {
    ($type: tt, $tval: tt) => {
        impl EndianData<$tval> for $type {
            #[inline(always)]
            fn value(&self) -> $tval {
                if arch_is_big_endian!() {
                    self.0 // keep
                } else {
                    self.0.to_be() // reverse
                }
            }
        }
    };
}

/// Implement an [EndianData<T>] for a specific type, and explain the data in little endianess
macro_rules! impl_converter_little {
    ($type: tt, $tval: tt) => {
        impl EndianData<$tval> for $type {
            #[inline(always)]
            fn value(&self) -> $tval {
                if arch_is_big_endian!() {
                    self.0.to_le() // reverse
                } else {
                    self.0
                }
            }
        }
    };
}

impl_converter_big!(BigEndian8, u8);
impl_converter_big!(BigEndian16, u16);
impl_converter_big!(BigEndian32, u32);
impl_converter_big!(BigEndian64, u64);

impl_converter_little!(LittleEndian8, u8);
impl_converter_little!(LittleEndian16, u16);
impl_converter_little!(LittleEndian32, u32);
impl_converter_little!(LittleEndian64, u64);
