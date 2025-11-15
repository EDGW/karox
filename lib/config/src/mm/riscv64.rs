// Some RISC-V Architecture specified memory-management configs and definitions

use crate::build_flags::{KERNEL_ENTRY_ADDR, KERNEL_SPACE_OFFSET};
use bitflags::bitflags;

// region: Class declarations and static properties for Page Table

    /// The count of the [PageTableEntry] array of a valid [PageTable]
    pub const PTABLE_LENGTH: usize = 512;

    /// Page offset bit width for Sv39 (12 bits = 4KiB pages)
    pub const PAGE_WIDTH: usize = 12;

    /// VPN section bit width for Sv39 (9 bits per level)
    pub const VPN_SECTION_WIDTH: usize = 9;

    /// This is the mask used to obtain the valid bits of a Sv39 address,
    /// used for calculating the index of a Sv39 virtual address in a page table entry.
    ///
    /// # Sv39 Virtual Address Format (39-bit)
    /// ```
    /// 38       30 29       21 20       12 11          0
    /// +----------+-----------+-----------+------------+
    /// | VPN[2]   | VPN[1]   | VPN[0]   | page offset |
    /// +----------+-----------+-----------+------------+
    ///    9 bits     9 bits     9 bits      12 bits
    /// ```
    /// 
    /// # Sv39 Physical Address Format (56-bit)
    /// ```
    /// 55       30 29       21 20       12 11          0
    /// +----------+-----------+-----------+------------+
    /// | PPN[2]   | PPN[1]   | PPN[0]   | page offset |
    /// +----------+-----------+-----------+------------+
    ///   26 bits     9 bits     9 bits      12 bits
    /// ```
    /// 
    /// # Address Translation Process
    /// Virtual Address → Page Table Walk → Physical Address
    /// ```
    /// VA[38:30] → Level 2 PTE → VA[29:21] → Level 1 PTE → VA[20:12] → Level 0 PTE → PA
    ///    VPN[2]                 VPN[1]                 VPN[0]
    /// ```
    pub const SV39_ADDR_MASK: usize = (1<<39)-1;

    /// Shift to extract VPN[2] (page offset + VPN[0] + VPN[1])
    pub const VPN2_SHIFT: usize = PAGE_WIDTH + VPN_SECTION_WIDTH + VPN_SECTION_WIDTH;


    /// 64-bit page table entry for Sv39 virtual memory system
    /// 
    /// Page table entry format in Sv39:
    /// ```
    /// 63      54 53        28 27        19 18         10 9   8 7 6 5 4 3 2 1 0
    /// +----------+------------+------------+------------+-----+-+-+-+-+-+-+-+-+
    /// | Reserved |   PPN[2]   |   PPN[1]   |   PPN[0]   | RSW |D|A|G|U|X|W|R|V|
    /// +----------+------------+------------+------------+-----+-+-+-+-+-+-+-+-+
    ///   10 bits    26 bits       9 bits       9 bits     2 bs  1 1 1 1 1 1 1 1
    /// ```
    pub type PageTableEntry = u64;

    bitflags! {
        /// Page table entry flags for Sv39 virtual memory system
        pub struct PTableEntryFlags: u64 {
            /// Valid - indicates the PTE is valid
            const V = 1 << 0;
            /// Readable - page can be read
            const R = 1 << 1;
            /// Writable - page can be written
            const W = 1 << 2;
            /// eXecutable - page can be executed
            const X = 1 << 3;
            /// User - accessible from user mode
            const U = 1 << 4;
            /// Global - globally mapped across address spaces
            const G = 1 << 5;
            /// Accessed - page has been accessed
            const A = 1 << 6;
            /// Dirty - page has been written to
            const D = 1 << 7;
            
            // Common permission combinations
            /// Page Directory Entry
            const PDIR = Self::V.bits();
            /// Read-only page
            const RONLY = Self::R.bits() | Self::V.bits();
            /// Read-write page
            const RW = Self::R.bits() | Self::W.bits() | Self::V.bits();
            /// Execute-only page
            const XONLY = Self::X.bits() | Self::V.bits();
            /// Read-execute page
            const RX = Self::R.bits() | Self::X.bits() | Self::V.bits();
            /// Read-write-execute page
            const RWX = Self::R.bits() | Self::W.bits() | Self::X.bits() | Self::V.bits();
        }
    }

    /// Create a simple page table entry from the physical address and the PTE properties.
    #[macro_export]
    macro_rules! create_pde {
        ($phyaddr:expr, $props: expr) => {
            {
                let ppn = ($phyaddr) >> $crate::mm::PAGE_WIDTH;
                ((ppn << 10) as u64 | ($props).bits()) as $crate::mm::PageTableEntry
            }
        };
    }

    /// A page table structure that takes a full page(4KiBs).
    pub type PageTable = [PageTableEntry; PTABLE_LENGTH];

// endregion

// region: Definitions for `satp` CSR Register

    bitflags! {
        /// SATP (Supervisor Address Translation and Protection) register modes
        pub struct SatpModes: u64 {
        /// No translation or protection - Bare mode
        /// Direct physical memory access
        const BARE = 0;
        
        /// Sv39 virtual memory system - 39-bit virtual address space
        /// 3-level page table with 4KiB pages
        const SV39 = 8;
        
        /// Sv48 virtual memory system - 48-bit virtual address space  
        /// 4-level page table with 4KiB pages
        const SV48 = 9;
        
        /// Sv57 virtual memory system - 57-bit virtual address space
        /// 5-level page table with 4KiB pages
        const SV57 = 10;
        
        /// Sv64 virtual memory system - 64-bit virtual address space
        /// 6-level page table with 4KiB pages
        const SV64 = 11;
    }
    }

    /// Create a valid value for `satp` CSR Register
    /// 
    /// SATP register layout:
    /// ```
    /// 63      60 59                  44 43                                0
    /// +----------+----------------------+----------------------------------+
    /// |   MODE   |         ASID         |                PPN               |
    /// +----------+----------------------+----------------------------------+
    ///   4 bits          16 bits                      44 bits
    /// ```
    /// 
    /// There are only 44 bits for PPN, which seemes to be not enough.
    /// However, considering that in fact we use Sv39, which only support 56-bits physical address (no more than 65536 TiBs physical memory),
    /// The 44 bits PPN width is **just enough**.
    #[macro_export]
    macro_rules! create_satp{
        ($mode: expr, $asid: expr, $ptable_ppn: expr)=>{
            {
                (($mode).bits() << 60) | (($asid) << 44) | ($ptable_ppn) as u64
            }
        }
    }
// endregion

// region: Kernel parameters

    /// VPN[2] index for kernel space base
    pub const KERNEL_SPACE_VPN2: usize = (SV39_ADDR_MASK & KERNEL_SPACE_OFFSET) >> VPN2_SHIFT;

    /// VPN[2] index for kernel entry point  
    pub const KERNEL_ENTRY_VPN2: usize = (SV39_ADDR_MASK & KERNEL_ENTRY_ADDR) >> VPN2_SHIFT;

// endregion