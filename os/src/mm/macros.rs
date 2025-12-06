
#[macro_export]
macro_rules! define_struct_copy_aligned {
    ($name: ident, $type: ty, $align: expr) => {
        #[derive(Copy,Clone)]
        #[repr(align($align))]
        #[repr(C)]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name{
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl $name{
            pub const fn from_value(value: $type) -> Self{
                Self(value)
            }
        }
    };
}

#[macro_export]
macro_rules! define_struct_copy {
    ($name: ident, $type: ty) => {
        #[derive(Copy,Clone)]
        #[repr(C)]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name{
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl $name{
            pub const fn from_value(value: $type) -> Self{
                Self(value)
            }
        }
    };
}

#[macro_export]
macro_rules! define_struct {
    ($name: ident, $type: ty) => {
        #[repr(C)]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name{
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

#[macro_export]
macro_rules! define_struct_aligned {
    ($name: ident, $type: ty, $align: expr) => {
        #[repr(C)]
        #[repr(align($align))]
        pub struct $name(pub $type);
        impl core::ops::Deref for $name{
            type Target = $type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
