//! This module defines some macros to pack basic types as structs with extra attributes

/// Define as a struct implementing [Copy] and aligned to specific bits.
#[macro_export]
macro_rules! define_struct_copy_aligned {
    ($name: ident, $type: ty, $align: expr) => {
        #[allow(missing_docs)]
        #[derive(Copy, Clone)]
        #[repr(align($align))]
        #[repr(C)]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl $name {
            /// Create the packed type from the inner value
            pub const fn from_value(value: $type) -> Self {
                Self(value)
            }
        }
    };
}

/// Define as a struct implementing [Copy].
#[macro_export]
macro_rules! define_struct_copy {
    ($name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[derive(Copy, Clone)]
        #[repr(C)]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl $name {
            /// Create the packed type from the inner value
            pub const fn from_value(value: $type) -> Self {
                Self(value)
            }
        }
    };
}

/// Define as a struct.
#[macro_export]
macro_rules! define_struct {
    ($name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[repr(C)]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

/// Define as a struct aligned to specific bits.
#[macro_export]
macro_rules! define_struct_aligned {
    ($name: ident, $type: ty, $align: expr) => {
        #[allow(missing_docs)]
        #[repr(C)]
        #[repr(align($align))]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
