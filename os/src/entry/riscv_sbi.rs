//! The entry point of the operating system on risc-v
//!
//! Initialization Steps:
//! 1. starting from [_start]: set up a temporary stack pointer.
//! 2. jumping to [adjust_dtb]: adjust dtb: copy the dtb to the lower space of the memory to
//!     avoid memory holes and virtual address overflow
//! 3. jumping to [setup]: set up a boot page table and the kernel stack.
//! 4. jumping to [start]: clear the bss and create a fdt tree instance (uninitialized).
//! 5. jumping to [crate::rust_main]

use crate::{
    arch::{
        KERNEL_OFFSET, PAGE_WIDTH,
        hart::{store_hart_id, wake_slave_harts},
        mm::{BOOT_STACK, paging::BOOT_PTABLE},
        symbols::_ekernel,
    },
    debug_ex,
    dev::{get_working_harts, info::dt::register_all},
    early_init_main,
    entry::shared::clear_bss,
    kernel_main, kernel_slave,
    mm::config::KERNEL_STACK_SHIFT,
    panic_init, phys_addr_from_symbol,
};
use core::{arch::naked_asm, ptr::copy};
use dt::fdt::reader::FdtReader;
use riscv::register::satp;
use utils::endian::EndianData;

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
        boot_stack_p = sym BOOT_STACK,
        boot_stack_shift = const KERNEL_STACK_SHIFT,
        copy_dtb = sym adjust_dtb,
    )
}

/// Move the dtb to the place right behind the kernel.
///
/// Do nothing if starting from a slave hart.
#[inline(always)]
fn adjust_dtb(hart_id: usize, dtb_addr: usize) -> ! {
    if dtb_addr != 0 {
        // Main Hart
        let reader = FdtReader::new(dtb_addr as *const u8);
        let len = reader.get_header().totalsize.value() as usize;

        let src = dtb_addr as *const u8;
        let dst = _ekernel as *const u8 as *mut u8;
        unsafe {
            copy(src, dst, len);
            setup(hart_id, dst as usize);
        }
    } else {
        // Slave Hart
        unsafe {
            setup(hart_id, 0);
        }
    }
}

/// Build an early boot table and set up the stack, and jumping to [start]
#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn setup(hart_id: usize, dtb_addr: usize) -> ! {
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
        boot_stack_p = sym BOOT_STACK,
        boot_stack_shift = const KERNEL_STACK_SHIFT,
        //boot_stack_size  = const KERNEL_STACK_SIZE,
        boot_satp = const BOOT_SATP,
        boot_table_addr = sym BOOT_PTABLE,
        page_width = const PAGE_WIDTH,
        offset = const KERNEL_OFFSET,
        start = sym start,
    )
}

/// Write the hart info to `tp` register and jump to kernel main.
fn start(hart_id: usize, dtb_addr: usize) -> ! {
    store_hart_id(hart_id);
    if dtb_addr != 0 {
        // Main Hart
        start_main(hart_id, dtb_addr);
    } else {
        // Slave Hart
        debug_ex!("karox RISC-V slave entry(hart: #{}).", hart_id);
        kernel_slave();
    }
}

fn start_main(hart_id: usize, dtb_addr: usize) -> ! {
    clear_bss();
    early_init_main();
    debug_ex!(
        "karox RISC-V main entry(hart: #{}, dtb: {:#x}).",
        hart_id,
        dtb_addr
    );

    let mut reader = FdtReader::new(dtb_addr as *const u8);
    let dev_tree = reader
        .read()
        .unwrap_or_else(|err| panic_init!("Error loading FDT: {:?}", err));

    register_all(dev_tree);

    for hart in get_working_harts() {
        if hart.hart_id == hart_id {
            continue;
        }
        wake_slave_harts(hart.hart_id, phys_addr_from_symbol!(_start));
    }

    kernel_main();
}
