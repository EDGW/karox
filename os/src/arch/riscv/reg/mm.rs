use bitflags::bitflags;

use crate::{arch::mm::paging::PageNum, define_bits_value, define_prop_bits, define_struct};

bitflags! {
    /// Paging modes used in the `satp` register.
    pub struct CrSatpModes : u8{
        /// Non-paging mode.
        const BARE  = 0;
        /// SV39 paging strategy.
        const SV39  = 8;
    }
}

// Represents a packed `satp` register value.
//
// The `satp` register layout:
// 63   60 59    44 43                     0
// | MODE |  ASID  |         PPN           |
define_struct!(num, CrSatpValue, usize);
impl CrSatpValue {
    /// Creates a new `satp` register value.
    ///
    /// # Arguments
    ///
    /// * `mode` - The paging mode.
    /// * `asid` - The address space identifier.
    /// * `ppn` - The physical page number.
    pub const fn create(mode: CrSatpModes, asid: u16, ppn: PageNum) -> CrSatpValue {
        CrSatpValue(((mode.bits as usize) << 60) | ((asid as usize) << 44) | ppn.0)
    }

    define_bits_value! {
        property(bitflags, mode, CrSatpModes, u8, 60, 64);
        property(num, asid, u16, 44, 60);
        property(packed_num, ppn, PageNum, 0, 44);
    }
}

// Defines the `satp` control register.
define_cr_register!(CrSatp, "satp", CrSatpValue, "sfence.vma");
