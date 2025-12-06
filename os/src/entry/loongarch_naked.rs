use core::{
    arch::{global_asm, naked_asm},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    arch::{
        PrvLevelBits,
        mm::{
            KERNEL_STACK, MemAccessType,
            config::{KERNEL_SPACE_OFFSET, KERNEL_STACK_SIZE, MMIO_OFFSET},
            paging::CrDMWValue,
        },
        reg::{CR_CPUID, CR_CRMD, CR_DMW0, CR_DMW1, CR_DMW2, CR_PRMD},
    },
    devices::device_tree::FdtTree,
    entry::shared::clear_bss,
    rust_main,
};


const BOOT_DMW0: CrDMWValue = CrDMWValue::create(
    PrvLevelBits::PLV0,
    MemAccessType::STRONG_NONCACHE,
    MMIO_OFFSET,
);
const BOOT_DMW1: CrDMWValue = CrDMWValue::create(
    PrvLevelBits::PLV0,
    MemAccessType::CACHE,
    KERNEL_SPACE_OFFSET,
);
const BOOT_DMW2: CrDMWValue = CrDMWValue::create(PrvLevelBits::PLV0, MemAccessType::CACHE, 0);

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start(hart_id: usize, dtb_addr: usize) {
    naked_asm!(
        // Enable mapped address translation
        "
            li.d    $t0, {boot_dmw0}
            csrwr   $t0, {cr_dmw0}
            
            li.d    $t0, {boot_dmw1}
            csrwr   $t0, {cr_dmw1}

            li.d    $t0, {boot_dmw2}
            csrwr   $t0, {cr_dmw2}

            li.w    $t0, 0xb0   // IE=0, PLV=0, DA=0, PG=1
            csrwr   $t0, {cr_crmd}
            csrwr   $t0, {cr_prmd}
        ",
        // Set up stack
        "
            la.global   $sp, {boot_stack}
            csrrd       $t0, {cr_cpuid}
            li.d        $t1, {stack_size}
            addi.d      $t0, $t0, 0x1
            mul.d       $t0, $t0, $t1
            add.d       $sp, $sp, $t0
        ",
        // Jump
        "
            csrrd       $a0, {cr_cpuid} // arg0: hart_id
            la.global   $t0, {start}
            jirl        $zero,$t0,0

        ",
        boot_dmw0 = const BOOT_DMW0.0,
        boot_dmw1 = const BOOT_DMW1.0,
        boot_dmw2 = const BOOT_DMW2.0,
        cr_dmw0 = const CR_DMW0,
        cr_dmw1 = const CR_DMW1,
        cr_dmw2 = const CR_DMW2,
        cr_crmd = const CR_CRMD,
        cr_prmd = const CR_PRMD,
        cr_cpuid = const CR_CPUID,
        boot_stack = sym KERNEL_STACK,
        stack_size = const KERNEL_STACK_SIZE,
        start = sym start

    )
}

unsafe extern "C" {
    pub unsafe fn _dtb();
}

global_asm! {
    "
        .section .data
        .align 4
        .globl dtb
        _dtb:
        .incbin \"runtime/qemu-loongarch64.dtb\"
    "
}

fn start(hart_id: usize) -> ! {
    clear_bss();
    let dev_tree = FdtTree::from_ptr(_dtb as *const u8);
    rust_main(hart_id, dev_tree);
}
