//! Karox Operating System Kernel
#![deny(warnings)]
#![deny(missing_docs)]
#![no_std]
#![no_main]

use core::{panic::PanicInfo};

use config::mm::_ekernel;
use fdt_resolver::{FdtPtr, structure::{enumerate_subnodes, memory::{get_memory_range}}};
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

    // Load Device Tree
    let root = fdt.load(pool).unwrap();
    enumerate_subnodes(root, |fullname, subnode|{
        let name = subnode.get_basic_name();
        match name {
            "memory" => {
                unsafe{
                    kprintln!("basic [{}]@{} dtype [{}]@{}",
                    name,
                    name.len()
                    ,subnode.get_prop("device_type").unwrap().value_as_str()
                    ,subnode.get_prop("device_type").unwrap().value_as_str().len());
                }
                let reg = get_memory_range(subnode).unwrap();
                kprintln!("Memory {:#x} - {:#x}",reg.start, reg.end);
            },
            _ => {
                kprintln!("Skipped node {}",fullname);
            }
        }
    });
    kprintln!("Device Tree Loaded.");
}

/// The panic handler
#[panic_handler]
pub fn panic_handler(_pinfo: &PanicInfo) -> !{
    kprintln!("[PANIC] {}", _pinfo);
    loop{
        
    }
}