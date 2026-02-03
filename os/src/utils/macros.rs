//! Macros for defining packed structs with various attributes and behaviors.

#[macro_export]
macro_rules! impl_basic {
    ($name: ident, $type: ty) => {
        impl core::convert::From<$type> for $name {
            fn from(value: $type) -> Self {
                $name { inner: value }
            }
        }
        impl core::convert::Into<$type> for $name {
            fn into(self) -> $type {
                self.inner
            }
        }

        impl core::ops::Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
        impl $name{
            pub const fn from_const(value: $type) -> Self {
                $name { inner: value }
            }
            pub const fn into_const(self) -> $type {
                self.inner
            }
        }
    };
}

#[macro_export]
macro_rules! impl_counter {
    ($name: ident, $type: ty) => {
        // A + N = A
        impl core::ops::Add<$type> for $name {
            type Output = $name;
            fn add(self, rhs: $type) -> Self::Output {
                $name {
                    inner: self.inner + rhs,
                }
            }
        }

        // N + A = A
        impl core::ops::Add<$name> for $type {
            type Output = $name;
            fn add(self, rhs: $name) -> Self::Output {
                $name {
                    inner: (self + rhs.inner),
                }
            }
        }

        /// A - A = N
        impl core::ops::Sub for $name {
            type Output = $type;
            fn sub(self, rhs: Self) -> Self::Output {
                self.inner - rhs.inner
            }
        }

        /// A - N = A
        impl core::ops::Sub<$type> for $name {
            type Output = $name;
            fn sub(self, rhs: $type) -> Self::Output {
                $name {
                    inner: (self.inner - rhs),
                }
            }
        }

        impl core::iter::Step for $name {
            fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                if start > end {
                    (0, None)
                } else {
                    (end.inner - start.inner, Some(end.inner - start.inner))
                }
            }

            fn forward_checked(start: Self, count: usize) -> Option<Self> {
                Some($name {
                    inner: (start.inner + count),
                })
            }

            fn backward_checked(start: Self, count: usize) -> Option<Self> {
                Some($name {
                    inner: (start.inner - count),
                })
            }
        }
    };
}

#[macro_export]
macro_rules! impl_number {
    ($name: ident, $type: ty) => {
        $crate::impl_counter!($name, $type);

        // A + A = A
        impl core::ops::Add for $name {
            type Output = $name;
            fn add(self, rhs: Self) -> Self::Output {
                $name {
                    inner: self.inner + rhs.inner,
                }
            }
        }

        /// N - A = A
        impl core::ops::Sub<$name> for $type {
            type Output = $name;
            fn sub(self, rhs: $name) -> Self::Output {
                $name {
                    inner: self - rhs.inner,
                }
            }
        }

        /// A * N = A
        impl core::ops::Mul<$type> for $name {
            type Output = $name;
            fn mul(self, rhs: $type) -> Self::Output {
                $name {
                    inner: self.inner * rhs,
                }
            }
        }

        /// A / N = A
        impl core::ops::Div<$type> for $name {
            type Output = $name;
            fn div(self, rhs: $type) -> Self::Output {
                $name {
                    inner: self.inner / rhs,
                }
            }
        }

        /// N * A = A
        impl core::ops::Mul<$name> for $type {
            type Output = $name;
            fn mul(self, rhs: $name) -> Self::Output {
                $name {
                    inner: rhs.inner * self,
                }
            }
        }
    };
}

/// Defines a packed struct with [Deref], [DerefMut], and utility methods.
/// Supports variants: copy_aligned, num, copy, aligned, or default.
#[macro_export]
macro_rules! define_struct {
    (copy_aligned, $name: ident, $type: ty, $align: expr) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone)]
        #[repr(align($align))]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name {
            inner: $type,
        }

        $crate::impl_basic!($name, $type);
    };

    (counter, $name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name {
            inner: $type,
        }

        $crate::impl_basic!($name, $type);
        $crate::impl_counter!($name, $type);
    };

    (number, $name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name {
            inner: $type,
        }

        $crate::impl_basic!($name, $type);
        $crate::impl_number!($name, $type);
    };

    (copy, $name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone)]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name {
            inner: $type,
        }

        $crate::impl_basic!($name, $type);
    };

    ($name: ident, $type: ty) => {
        #[allow(missing_docs)]
        #[derive(Debug)]
        #[repr(C)]
        #[non_exhaustive]
        pub struct $name {
            inner: $type,
        }

        $crate::impl_basic!($name, $type);
    };

    (aligned, $name: ident, $type: ty, $align: expr) => {
        #[allow(missing_docs)]
        #[derive(Debug)]
        #[repr(C)]
        #[repr(align($align))]
        #[non_exhaustive]
        pub struct $name {
            inner: $type,
        }
        $crate::impl_basic!($name, $type);
    };
}
