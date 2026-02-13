//! Module for Device Tree

use crate::{
    arch::symbols::{_ekernel, _skernel},
    debug_ex,
    dev::{
        DEVICE_ROOT, Device, DeviceInfo, DeviceType, DriverStatus, GENERAL_MEM, IntcInfo, IntrInfo,
        MemorySet,
        handle::{Handle, HandleRef},
        mmio::IoRange,
        register_hart,
    },
    panic_init, phys_addr_from_symbol,
};
use alloc::{boxed::Box, vec};
use dt::node::{DeviceTree, Node, NodeType};
use log::warn;
use spin::RwLock;

pub fn register_all(dev_tree: DeviceTree) {
    register_mem(&dev_tree);
    register_harts(&dev_tree);
    register_devices(&dev_tree);
}

fn register_mem(dev_tree: &DeviceTree) {
    debug_ex!("Registering memory info...");
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
    debug_ex!("Memory info registered.");
}

fn register_harts(dev_tree: &DeviceTree) {
    debug_ex!("Registering hart info...");
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
        debug_ex!("\tHart #{}, status \"{}.\"", hart_id, status);
        if status == "okay" {
            register_hart(hart_id);
        } else {
            log::warn!(
                "CPU #{:} is not initialized properly: status {:}.",
                hart_id,
                status
            );
        }
    }
    debug_ex!("Hart info registered.");
}

fn register_devices(dev_tree: &DeviceTree) {
    debug_ex!("Registering devices...");
    register_devices_by_node(
        DEVICE_ROOT.create_ref(),
        &dev_tree,
        dev_tree.get_node("/").unwrap_or_else(|| {
            panic_init!("Error registering devices: Unable to fetch the root node.");
        }),
    );
    debug_ex!("Devices registered.");
}

fn register_devices_by_node(dev: HandleRef<Device>, dev_tree: &DeviceTree, node: &Node) {
    let handle: Handle<Device> = dev.get_handle().unwrap_or_else(|| {
        panic_init!(
            "Error registering devices under node '{}': The parent device is not available.",
            node.full_name
        )
    });
    for child in dev_tree.get_children(node) {
        if child.node_type == NodeType::Description {
            debug_ex!(
                "\tSkipped Description Node {}.",
                dev_tree.get_full_path(child)
            );
            continue;
        }
        // compatible list
        let mut comp_list = vec![];
        let comp_prop = dev_tree.get_property(child, "compatible");
        if let Some(prop) = comp_prop {
            match prop.value_as_strlist() {
                Err(err) => {
                    warn!(
                        "Error loading device '{}': Unable to get 'compatible' property: {:?}",
                        dev_tree.get_full_path(child),
                        err
                    );
                    continue;
                }
                Ok(values) => {
                    for val in values {
                        comp_list.push(Box::from(val));
                    }
                }
            }
        }
        // device type
        let mut dev_type = DeviceType::UNSPECIFIED;
        let mut intc_info = None;
        if let Some(_) = dev_tree.get_property(child, "interrupt-controller") {
            dev_type |= DeviceType::INTC;
            if let Some(prop) = dev_tree.get_property(child, "phandle") {
                match prop.value_as_u32() {
                    Ok(value) => {
                        intc_info = Some(IntcInfo {
                            intc_id: value as usize,
                        })
                    }
                    Err(err) => {
                        warn!(
                            "Error loading device '{}' as intr handle: Unable to parse 'phandle' property: {:?}",
                            dev_tree.get_full_path(child),
                            err
                        );
                    }
                }
            }
        };
        if let Some(_) = comp_prop {
            dev_type |= DeviceType::DEVICE;
        }
        // mmio info
        let mut io_addr = vec![];
        if let Ok(reg) = dev_tree.get_reg_value(child) {
            for sec in reg {
                io_addr.push(IoRange::from(sec));
            }
        }

        // intr info
        let mut intr_info = vec![];
        match dev_tree.get_intr_info(child) {
            Err(err) => {
                warn!(
                    "Error loading device '{}': Unable to get properties for interrupt info: {:?}",
                    dev_tree.get_full_path(child),
                    err
                );
                continue;
            }
            Ok(info) => {
                for (intc, ir) in info {
                    intr_info.push(IntrInfo { intc_id: intc, irq_id: ir });
                }
            }
        }
        // add device
        let child_dev = handle.new_child(DeviceInfo {
            name: Box::from(child.full_name.as_ref()),
            comp_list,
            dev_type,
            drv_stat: RwLock::new(DriverStatus::Unrecognized),
            io_addr,
            intr_info,
            intc_info,
        });
        match child_dev.add() {
            Err(err) => warn!(
                "Error adding device '{}' to its parent device: {:?}",
                dev_tree.get_full_path(child),
                err
            ),
            Ok(dev_ref) => {
                debug_ex!(
                    "\tRegistered device {} [{:?}].",
                    dev_tree.get_full_path(child),
                    dev_ref.get_handle().unwrap().info.dev_type
                );
                register_devices_by_node(dev_ref, dev_tree, child);
            }
        }
    }
}
