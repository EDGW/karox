//! Macros for defining packed structs with various attributes and behaviors.

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
            /// Creates new instance from value.
            pub const fn from_value(value: $type) -> Self {
                Self(value)
            }
            /// Gets inner value.
            pub const fn get_value(&self) -> $type {
                self.0
            }
            /// Sets inner value.
            pub const fn set_value(&mut self, value: $type) {
                self.0 = value;
            }
        }
    };

    (num, $name: ident, $type: ty) => {
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
            /// Creates new instance from value.
            pub const fn from_value(value: $type) -> Self {
                Self(value)
            }
            /// Gets inner value.
            pub const fn get_value(&self) -> $type {
                self.0
            }
            /// Sets inner value.
            pub const fn set_value(&mut self, value: $type) {
                self.0 = value;
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

    (copy, $name: ident, $type: ty) => {
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
            /// Creates new instance from value.
            pub const fn from_value(value: $type) -> Self {
                Self(value)
            }
            /// Gets inner value.
            pub const fn get_value(&self) -> $type {
                self.0
            }
            /// Sets inner value.
            pub const fn set_value(&mut self, value: $type) {
                self.0 = value;
            }
        }
    };

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

    (aligned, $name: ident, $type: ty, $align: expr) => {
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

/// Defines bit-field properties with getter, setter, and modifier methods.
/// Supports variants: bitflags, num, packed_num.
#[macro_export]
macro_rules! define_prop_bits {
    (bitflags, $prop_name: ident, $prop_type: tt, $bitflag_type: tt, $st:expr, $ed:expr) => {
        paste::paste!{
            /// Bit mask for property.
            pub const [<PROP_ $prop_name:upper _MASK>]: usize = (((1 as usize)<<($ed-1))+(((1 as usize)<<($ed-1))-1)) - (((1 as usize)<<($st as usize))-1);

            /// Gets property value.
            pub fn [<get_ $prop_name>](&self) -> $prop_type{
                $prop_type{bits:(((&self.get_value() & Self::[<PROP_ $prop_name:upper _MASK>]) >> $st)) as $bitflag_type}
            }

            /// Sets property value.
            pub fn [<set_ $prop_name>](&mut self,value: $prop_type){
                let val = ((value.bits as usize) << ($st as usize)) & Self::[<PROP_ $prop_name:upper _MASK>];
                let mut reg = self.get_value();
                reg &= !Self::[<PROP_ $prop_name:upper _MASK>];
                reg |= val;
                self.set_value(reg);
            }

            /// Modifies property and returns new instance.
            pub fn [<with_ $prop_name>](&mut self,value: $prop_type) -> Self{
                let val = ((value.bits as usize) << ($st as usize)) & Self::[<PROP_ $prop_name:upper _MASK>];
                let mut reg = self.get_value();
                reg &= !Self::[<PROP_ $prop_name:upper _MASK>];
                reg |= val;
                Self::from_value(reg)
            }
        }
    };

    (num, $prop_name: ident, $prop_type: tt, $st:expr, $ed:expr) => {
        paste::paste!{
            /// Bit mask for property.
            pub const [<PROP_ $prop_name:upper _MASK>]: usize = (((1 as usize)<<($ed-1))+(((1 as usize)<<($ed-1))-1)) - (((1 as usize)<<($st as usize))-1);

            /// Gets property value.
            pub fn [<get_ $prop_name>](&self) -> $prop_type{
                (((&self.get_value() & Self::[<PROP_ $prop_name:upper _MASK>]) >> $st)) as $prop_type
            }

            /// Sets property value.
            pub fn [<set_ $prop_name>](&mut self,value: $prop_type){
                let val = ((value as usize) << ($st as usize)) as usize & Self::[<PROP_ $prop_name:upper _MASK>];
                let mut reg = self.get_value();
                reg &= !Self::[<PROP_ $prop_name:upper _MASK>];
                reg |= val;
                self.set_value(reg);

            }

            /// Modifies property and returns new instance.
            pub fn [<with_ $prop_name>](&mut self,value: $prop_type) -> Self{
                let val = ((value as usize) << ($st as usize)) as usize & Self::[<PROP_ $prop_name:upper _MASK>];
                let mut reg = self.get_value();
                reg &= !Self::[<PROP_ $prop_name:upper _MASK>];
                reg |= val;
                Self::from_value(reg)
            }
        }
    };

    (packed_num, $prop_name: ident, $prop_type: tt, $st:expr, $ed:expr) => {
        paste::paste!{
            /// Bit mask for property.
            pub const [<PROP_ $prop_name:upper _MASK>]: usize = (((1 as usize)<<($ed-1))+(((1 as usize)<<($ed-1))-1)) - (((1 as usize)<<($st as usize))-1);

            /// Gets property value.
            pub fn [<get_ $prop_name>](&self) -> $prop_type{
                $prop_type::from_value((&self.get_value() & Self::[<PROP_ $prop_name:upper _MASK>]) >> $st)
            }

            /// Sets property value.
            pub fn [<set_ $prop_name>](&mut self,value: $prop_type){
                let val = ((value.get_value() as usize) << ($st as usize)) as usize & Self::[<PROP_ $prop_name:upper _MASK>];
                let mut reg = self.get_value();
                reg &= !Self::[<PROP_ $prop_name:upper _MASK>];
                reg |= val;
                self.set_value(reg);

            }

            /// Modifies property and returns new instance.
            pub fn [<with_ $prop_name>](&mut self,value: $prop_type) -> Self{
                let val = ((value.get_value() as usize) << ($st as usize)) as usize & Self::[<PROP_ $prop_name:upper _MASK>];
                let mut reg = self.get_value();
                reg &= !Self::[<PROP_ $prop_name:upper _MASK>];
                reg |= val;
                Self::from_value(reg)
            }
        }
    };
}

/// Defines multiple bit-field properties using [define_prop_bits].
#[macro_export]
macro_rules! define_bits_value {
    ( $( property($type:ident, $prop_name: ident, $prop_type: tt, $($args:tt),+) ; )* ) => {
        $(
            define_prop_bits!($type,$prop_name, $prop_type, $($args),+);
        )*
    };
}
