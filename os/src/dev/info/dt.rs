//! Module for Device Tree

use crate::{
    arch::symbols::{_ekernel, _skernel},
    dev::{GENERAL_MEM, MemorySet, register_hart},
    panic_init, phys_addr_from_symbol,
};
use dt::node::DeviceTree;

pub fn register_devs(dev_tree: DeviceTree) {
    register_mem(&dev_tree);
    register_harts(&dev_tree);
}

fn register_mem(dev_tree: &DeviceTree) {
    let mem_nodes = dev_tree.get_nodes("/memory");
    if mem_nodes.is_empty() {
        panic_init!("Node '/memory' not found in device tree.");
    }
    let rsv_nodes = dev_tree.get_nodes("/reserved-memory/*");
    let mut mem = MemorySet::new();
    for node in mem_nodes {
        for range in dev_tree.get_reg_value(node).unwrap_or_else(|err| {
            panic_init!("Error loading 'reg' value of node '/memory': {:?}.", err)
        }) {
            mem.add(range);
        }
    }
    for node in rsv_nodes {
        for range in dev_tree.get_reg_value(node).unwrap_or_else(|err| {
            panic_init!("Error loading 'reg' value of node '/memory': {:?}.", err)
        }) {
            mem.sub(range);
        }
    }
    for range in &dev_tree.mem_rsv_map {
        mem.sub(range.clone());
    }
    let self_range = phys_addr_from_symbol!(_skernel)..phys_addr_from_symbol!(_ekernel);
    mem.sub(self_range);
    GENERAL_MEM.call_once(|| mem);
}

fn register_harts(dev_tree: &DeviceTree) {
    let cpu_nodes = dev_tree.get_nodes("/cpus/cpu");
    for node in cpu_nodes {
        let reg_arr = dev_tree.get_reg_value(node).unwrap_or_else(|err| {
            panic_init!(
                "Error loading cpu info when trying to load 'reg' value: {:?}.",
                err
            )
        });
        if reg_arr.is_empty() {
            panic_init!("Error loading cpu info: empty 'reg' value in dtb.");
        }
        let hart_id = reg_arr[0].start;
        let status = dev_tree
            .get_property(node, "status")
            .unwrap_or_else(|| {
                panic_init!(
                    "Error loading cpu info: missing 'status' value of cpu #{:}.",
                    hart_id
                )
            })
            .value_as_str()
            .unwrap_or_else(|err| {
                panic_init!(
                    "Err loading cpu info: invalid 'sstatus' value of cpu #{:}: {:?}.",
                    hart_id,
                    err
                )
            });
        if status == "okay" {
            register_hart(hart_id);
        } else {
            log::warn!(
                "CPU #{:} is not initialized property: status {:}.",
                hart_id,
                status
            );
        }
    }
}
