//! Karox Operating System Kernel
#![deny(warnings)]
#![deny(missing_docs)]
#![no_std]
#![no_main]

use core::{panic::PanicInfo};

use config::mm::_ekernel;
use fdt_resolver::{FdtPtr, structure::{enumerate_subnodes}};
use mm::linear_pool::LinearPool;

#[macro_use]
pub mod entry;


/// The main function of the kernel, called from [entry::start]
/// 
/// Before this function executes, the stack space and a boot page table should be well-prepared.
pub unsafe fn kernel_main(hart_id: usize, fdt: FdtPtr) -> ! {
    let mut pool= LinearPool::from(_ekernel as *mut u8);
    kprintln!("Kernel started on hart #{}.",hart_id);
    load_fdt(fdt, &mut pool);
    loop {}
}

fn load_fdt(fdt: FdtPtr, pool: &mut LinearPool){
    // Validate
    if let Err(str) = fdt.validate()
    {
        panic!("Unable to read the device table: {}", str);
    }

    // Read Reserved Memory
    fdt.enumerate_rsvmem(|block|{
        kprintln!("Reserved Memory [{:#x} - {:#x}]",block.addr.value(),block.addr.value()+block.length.value());
    });

    // Load Device Tree
    let res = fdt.load(pool);
    if let Err(info) = res{
        kprintln!("Unable to load the fdt: {}", info);
    }
    let root = res.unwrap();
    enumerate_subnodes(root, |name, node|{
        kprintln!("Node {} with {} subnodes and {} properties.",name, node.children_cnt, node.props_cnt);
    });
    kprintln!("OK");
}

/// The panic handler
#[panic_handler]
pub fn panic_handler(_pinfo: &PanicInfo) -> !{
    kprintln!("[PANIC] {}", _pinfo);
    loop{
        
    }
}