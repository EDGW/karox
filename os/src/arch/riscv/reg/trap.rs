use bitflags::bitflags;

use crate::define_struct;

bitflags! {
    pub struct StVecMode : u8{

    }
}

define_struct!(num, CrStVec, usize);