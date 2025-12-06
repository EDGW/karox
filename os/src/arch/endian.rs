#[derive(Clone, Copy)]
pub struct BigEndian8(u8);
#[derive(Clone, Copy)]
pub struct LittleEndian8(u8);
#[derive(Clone, Copy)]
pub struct BigEndian16(u16);
#[derive(Clone, Copy)]
pub struct LittleEndian16(u16);
#[derive(Clone, Copy)]
pub struct BigEndian32(u32);
#[derive(Clone, Copy)]
pub struct LittleEndian32(u32);
#[derive(Clone, Copy)]
pub struct BigEndian64(u64);
#[derive(Clone, Copy)]
pub struct LittleEndian64(u64);

pub trait EndianData<T> : Copy + Clone{
    fn value(&self) -> T;
}

#[cfg(target_arch = "loongarch64")]
macro_rules! arch_is_big_endian {
    () => {
        false
    };
}
#[cfg(target_arch = "riscv64")]
macro_rules! arch_is_big_endian {
    () => {
        false
    };
}
#[cfg(not(any(target_arch = "loongarch64", target_arch = "riscv64")))]
macro_rules! arch_is_big_endian {
    () => {
        compile_error!("Unsupported architecture!");
    };
}

macro_rules! impl_converter_big {
    ($type: tt, $tval: tt) => {
        impl EndianData<$tval> for $type{
            #[inline(always)]
            fn value(&self) -> $tval{
                if arch_is_big_endian!(){
                    self.0  // keep
                }
                else
                {
                    self.0.to_be()  // reverse
                }
            } 
        }
    };
}
macro_rules! impl_converter_little {
    ($type: tt, $tval: tt) => {
        impl EndianData<$tval> for $type{
            #[inline(always)]
            fn value(&self) -> $tval{
                if arch_is_big_endian!(){
                    self.0.to_le()  // reverse
                }
                else
                {
                    self.0
                }
            } 
        }
    };
}

impl_converter_big!(BigEndian8,u8);
impl_converter_big!(BigEndian16,u16);
impl_converter_big!(BigEndian32,u32);
impl_converter_big!(BigEndian64,u64);

impl_converter_little!(LittleEndian8,u8);
impl_converter_little!(LittleEndian16,u16);
impl_converter_little!(LittleEndian32,u32);
impl_converter_little!(LittleEndian64,u64);