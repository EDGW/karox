//! This module provides macros for defining packed structs with various attributes and behaviors.

/// Defines a struct that implements [Copy] and is aligned to a specific number of bits.
#[macro_export]
macro_rules! define_struct_copy_aligned {
    ($name: ident, $type: ty, $align: expr) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone)]
        #[repr(align($align))]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl $name {
            /// Create the packed type from the inner value
            pub const fn from_value(value: $type) -> Self {
                Self(value)
            }
            /// Get the inner value
            pub const fn get_value(&self) -> $type {
                self.0
            }
        }
    };
}

/// Defines a struct that behaves like a number.
#[macro_export]
macro_rules! define_struct_num {
    ($name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl $name {
            /// Create the packed type from the inner value
            pub const fn from_value(value: $type) -> Self {
                Self(value)
            }
            /// Get the inner value
            pub const fn get_value(&self) -> $type {
                self.0
            }
        }

        impl core::ops::Add for $name {
            type Output = $name;
            fn add(self, rhs: Self) -> Self::Output {
                $name(self.0 + rhs.0)
            }
        }

        impl core::ops::Add<$type> for $name {
            type Output = $name;
            fn add(self, rhs: $type) -> Self::Output {
                $name(self.0 + rhs)
            }
        }

        impl core::ops::Add<$name> for $type {
            type Output = $name;
            fn add(self, rhs: $name) -> Self::Output {
                $name(self + rhs.0)
            }
        }

        impl core::ops::Sub for $name {
            type Output = $name;
            fn sub(self, rhs: Self) -> Self::Output {
                $name(self.0 - rhs.0)
            }
        }

        impl core::ops::Sub<$name> for $type {
            type Output = $name;
            fn sub(self, rhs: $name) -> Self::Output {
                $name(self - rhs.0)
            }
        }

        impl core::ops::Sub<$type> for $name {
            type Output = $name;
            fn sub(self, rhs: $type) -> Self::Output {
                $name(self.0 - rhs)
            }
        }

        impl core::ops::Mul<$type> for $name {
            type Output = $name;
            fn mul(self, rhs: $type) -> Self::Output {
                $name(self.0 * rhs)
            }
        }

        impl core::ops::Div<$type> for $name {
            type Output = $name;
            fn div(self, rhs: $type) -> Self::Output {
                $name(self.0 / rhs)
            }
        }

        impl core::ops::Mul<$name> for $type {
            type Output = $name;
            fn mul(self, rhs: $name) -> Self::Output {
                $name(rhs.0 * self)
            }
        }
    };
}

/// Defines a struct that implements [Copy].
#[macro_export]
macro_rules! define_struct_copy {
    ($name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone)]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl $name {
            /// Create the packed type from the inner value
            pub const fn from_value(value: $type) -> Self {
                Self(value)
            }
            /// Get the inner value
            pub const fn get_value(&self) -> $type {
                self.0
            }
        }
    };
}

/// Defines a basic struct.
#[macro_export]
macro_rules! define_struct {
    ($name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[derive(Debug)]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

/// Defines a struct aligned to a specific number of bits.
#[macro_export]
macro_rules! define_struct_aligned {
    ($name: ident, $type: ty, $align: expr) => {
        #[allow(missing_docs)]
        #[derive(Debug)]
        #[repr(C)]
        #[repr(align($align))]
        #[non_exhaustive]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name {
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}
