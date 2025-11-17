// The RISC-V Architecture implementation for the entry.

use core::arch::naked_asm;

use config::{build_flags::{KERNEL_PAGE_ADDR, KERNEL_SPACE_OFFSET}, create_pde, create_satp, mm::{KERNEL_ENTRY_VPN2, KERNEL_SPACE_VPN2, PAGE_WIDTH, PTABLE_LENGTH, PTableEntryFlags, PageTable, PageTableEntry, SatpModes}};
use fdt_resolver::{FdtPtr};

use crate::{kernel_main};

/// The Boot Page Table,
/// including 2 large 1GiB pages, mapping higher half kernel spaces to physical spaces.
/// 
/// This is only for early use.

static BOOT_PTABLE: PageTable = {
    let mut ptable: PageTable = PageTable{
        0: [0; PTABLE_LENGTH]
    };
    let ptablentry: PageTableEntry = create_pde!(KERNEL_PAGE_ADDR, PTableEntryFlags::RWX);
    ptable.0[KERNEL_ENTRY_VPN2] = ptablentry;  // 1GiB page
    ptable.0[KERNEL_SPACE_VPN2 + KERNEL_ENTRY_VPN2] = ptablentry;  // higher half 1GiB page
    ptable
};

/// The `mode` field of `satp` CSR Register
const BOOT_SATP_MODE: u64 = create_satp!(SatpModes::SV39, 0, 0);


/// The operating system entry point
/// 
/// Initialize the kernel stack and a boot page table, then jump to [start]
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start(){
    naked_asm!(
        // init boot ptable
        "
            la      t0, {boot_table_addr}
            srli    t0, t0, {page_width}
            li      t1, {boot_satp}
            or      t0, t0, t1
            csrw    satp, t0
            sfence.vma
            
        ",
        // init stack top
        "   la      sp, {boot_stack_p}
            li      t0, {boot_stack_size}
            add     sp, sp, t0
            addi    sp, sp, -0x100",
        // jump
        "   la      a2, {start}
            la      t0, {offset}
            or      a2, a2, t0
            jr      a2",
        // The hart id and dtb addr args are passed in reg a0 & a1
        boot_stack_p = sym KERNEL_STACK,
        boot_stack_size = const KERNEL_STACK_SIZE,
        boot_satp = const BOOT_SATP_MODE,
        boot_table_addr = sym BOOT_PTABLE,
        page_width = const PAGE_WIDTH,
        offset = const KERNEL_SPACE_OFFSET,
        start = sym start,
    )
}

/// The main function of the entry, after the ptable is initialized.
/// 
/// There are no other functions than jumping to the [crate::kernel_main] in this function.
/// 
/// We use this 'duplicated jump' to ensure that,
/// **before jumping to [crate::kernel_main], the registers would be well-initialized by the caller.**
pub fn start(hart_id: usize, dtb_addr: usize) -> !{
    let fdtptr = FdtPtr::from_addr(dtb_addr);
    unsafe{
        kernel_main(hart_id, fdtptr);
    }
}

/// Print strings to the serial port
pub fn kserial_output(s: &str){
    for c in s.bytes(){
        #[allow(deprecated)]
        sbi_rt::legacy::console_putchar(c as usize);
    }
}