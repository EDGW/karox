use core::arch::naked_asm;

use crate::{
    arch::mm::{
        KERNEL_STACK,
        config::{
            KERNEL_SPACE_OFFSET, KERNEL_STACK_SHIFT, /*KERNEL_STACK_SIZE,*/
            PAGE_WIDTH,
        },
        paging::{BOOT_PTABLE, CrSatpModes, CrSatpValue, PhysicalPageNum},
    },
    devices::device_tree::FdtTree,
    entry::shared::clear_bss,
    kserial_println, rust_main,
};

const BOOT_SATP: CrSatpValue = CrSatpValue::create(CrSatpModes::SV39, 0, PhysicalPageNum(0));

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start(hart_id: usize, dtb_addr: usize) {
    naked_asm!(
        // init boot ptable
        "
            la      t0, {boot_table_addr}       // > t0 = boot_table_addr << page_width
            srli    t0, t0, {page_width}        // |
            li      t1, {boot_satp}             // > t1 = boot_satp
            or      t0, t0, t1                  // > t0 = t0 | t1
            csrw    satp, t0                    // > satp = t0
            sfence.vma                          // > refresh
            
        ",
        // load offset
        "   la      t2, {offset}",
        // init stack top
        "   la      a2, {boot_stack_p}          // sp = boot_stack_p
            li      t0, {boot_stack_shift}      // t0 = boot_stack_shift
            addi    a0, a0, 1                   // hart_id += 1
            sll     t1, a0, t0                  // t1 = hart_id << t0
            addi    a2, a2, 0                   // sp -= 0
            add     a2, a2, t1                  // sp += t1
            addi    a0, a0, -1                  // hart_id -= 1
            or      a2, a2, t2
            mv      sp, a2
            ",            
        // jump
        "   la      a3, {start}
            or      a3, a3, t2
            jr      a3",
        // The hart id and dtb addr args are passed in reg a0 & a1
        boot_stack_p = sym KERNEL_STACK,
        boot_stack_shift = const KERNEL_STACK_SHIFT,
        //boot_stack_size  = const KERNEL_STACK_SIZE,
        boot_satp = const BOOT_SATP.0,
        boot_table_addr = sym BOOT_PTABLE,
        page_width = const PAGE_WIDTH,
        offset = const KERNEL_SPACE_OFFSET,
        start = sym start,
    )
}
fn start(hart_id: usize, dtb_ptr: usize) -> ! {
    clear_bss();
    kserial_println!("karox entry for RISC-V architecture.");
    kserial_println!("Kernel running on hart #{:#x}", hart_id);
    let dtree = FdtTree::from_ptr(dtb_ptr as *const u8);
    rust_main(hart_id, dtree);
}
