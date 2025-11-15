//! This module doesn't contain any compilable code.
//! It only provides macros to better organize different architecture-specific configurations.

/// Include architecture-specific files based on the target architecture.
/// This is not a sub-module declaration, but a macro to include files.
/// If provided with a namespace literal, it will include files from that directory.
#[macro_export]
macro_rules! include_arch_files {
    () => {
        include!(concat!($crate::arch_id!(),".rs"));
    };
    ($ns: literal) => {
        include!(concat!($ns,"/",$crate::arch_id!(),".rs"));
    };
}

/// Include architecture-specific files based on the target architecture.
/// This is a sub-module declaration, and it will include files from the specified directory,
/// and declare them as a sub-module.
#[macro_export]
macro_rules! submod_arch {
    ($ns: literal, $name: ident) => {
        mod $name{
            include_arch_files!($ns);
        }
    };
}

/// Get current architecture ID
#[macro_export]
#[cfg(target_arch = "riscv64")]
macro_rules! arch_id{
    () => {"riscv64"};
}
/// Get current architecture ID
#[macro_export]
#[cfg(target_arch = "loongarch64")]
macro_rules! arch_id{
    () => {"loongarch64"};

}
