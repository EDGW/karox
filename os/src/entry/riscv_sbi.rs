//! The entry point of the operating system on risc-v
//!
//! Initialization Steps:
//! 1. starting from [_start]: set up a temporary stack pointer.
//! 2. jumping to [adjust_dtb]: adjust dtb: copy the dtb to the lower space of the memory to avoid memory holes and virtual address overflow
//! 3. jumping to [setup]: set up a boot page table and the kernel stack.
//! 4. jumping to [start]: clear the bss and create a fdt tree instance (uninitialized).
//! 5. jumping to [crate::rust_main]

use core::arch::naked_asm;

use crate::{
    arch::{
        KERNEL_OFFSET, PAGE_WIDTH,
        endian::EndianData,
        mm::{KERNEL_STACK, paging::BOOT_PTABLE},
        symbols::_ekernel,
    },
    devices::device_info::FdtTree,
    entry::shared::clear_bss,
    mm::config::KERNEL_STACK_SHIFT,
    rust_main,
};
use riscv::register::satp;

/// Boot `satp` register value
///
/// The paging mod is set to SV39, and the ppn is set during [_start]
const BOOT_SATP: usize = (satp::Mode::Sv39 as usize) << 60;

/// The entry point of the operating system
///
/// Set a temporary stack, and jumping to [adjust_dtb].
#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start(hart_id: usize, dtb_addr: usize) {
    naked_asm!(
        // init stack top
        "   la      a2, {boot_stack_p}          // sp = boot_stack_p
            li      t0, {boot_stack_shift}      // t0 = boot_stack_shift
            addi    a0, a0, 1                   // hart_id += 1
            sll     t1, a0, t0                  // t1 = hart_id << t0
            addi    a2, a2, 0                   // sp -= 0
            add     a2, a2, t1                  // sp += t1
            addi    a0, a0, -1                  // hart_id -= 1
            mv      sp, a2
            ",            
        // jump
        "   la      a3, {copy_dtb}
            jr      a3",
        // The hart id and dtb addr args are passed in reg a0 & a1
        boot_stack_p = sym KERNEL_STACK,
        boot_stack_shift = const KERNEL_STACK_SHIFT,
        copy_dtb = sym adjust_dtb,
    )
}

/// Move the dtb to the place right behind the kernel.
#[inline(always)]
fn adjust_dtb(hart_id: usize, dtb_ptr: *const u8) -> ! {
    unsafe {
        let dtb = FdtTree::from_ptr(dtb_ptr);
        let len = dtb.get_header().totalsize.value() as usize;

        let src = dtb.fdt_ptr as *const u8;
        let dst = _ekernel as *const u8 as *mut u8;
        for i in 0..len {
            *(dst.add(i)) = *(src.add(i));
        }
        setup(hart_id, dst as *const u8);
    }
}

/// Build an early boot table and set up the stack, and jumping to [start]
#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn setup(hart_id: usize, dtb_ptr: *const u8) -> ! {
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
        boot_satp = const BOOT_SATP,
        boot_table_addr = sym BOOT_PTABLE,
        page_width = const PAGE_WIDTH,
        offset = const KERNEL_OFFSET,
        start = sym start,
    )
}

/// Step 2
fn start(hart_id: usize, dtb_ptr: usize) -> ! {
    clear_bss();
    let dtree = FdtTree::from_ptr(dtb_ptr as *const u8);
    rust_main(hart_id, dtree);
}
