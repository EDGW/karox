//! This module is used to resolve the structure block of a FDT

use core::{slice::{self}, str};

use config::mm::endian::{BigEndian32};


/// *The structure block is composed of a sequence of pieces, each beginning with a token,*
/// *that is, a big-endian 32-bit integer. Some tokens are followed by extra data,*
/// *the format of which is determined by the token value. All tokens shall be aligned on a 32-bit boundary,*
/// *which may require padding bytes (with a value of 0x0) to be inserted after the previous token’s data.*
/// 
/// This enum defines 5 basic token types
#[repr(u32)]
pub enum FdtNodeType{
    /// The token marks the beginning of a node’s representation.
    FdtBeginNode=   0x1,
    /// The token marks the end of a node’s representation.
    FdtEndNode  =   0x2,
    /// The token marks the beginning of the representation of one property in the devicetree.
    FdtProp     =   0x3,
    /// The token will be ignored by any program parsing the device tree.
    FdtNop      =   0x4,
    /// The token marks the end of the structure block.
    FdtEnd      =   0x9
}

/// This struct represents a node in the momory
pub struct NodeInfo{
    /// The name reference, which directly points to the name string area of the node in the memory
    pub name: &'static str,
    /// The pointer to the first property
    pub first_prop_ptr: *const BigEndian32,
    /// The count of the properties
    pub props_cnt: usize,
    /// The pointer to the first child
    pub first_child_ptr: *const NodeInfo,
    /// The count of the children
    pub children_cnt: usize,
    /// The pointer to the next node, if exists, or is undefined
    pub next_node: *const NodeInfo,
}

fn peek(ptr: *mut *const BigEndian32) -> u32{
    unsafe{
        (**ptr).value()
    }
}

fn read(ptr: *mut *const BigEndian32) -> u32{
    unsafe{
        let res = (**ptr).value();
        *ptr = (*ptr).add(1);
        res
    }
}

fn align4(ptr: *mut *const u8){
    unsafe{
        let addr = *ptr as usize;
        let res = (addr>>2)<<2;
        if res == addr{
            *ptr = addr as *const u8;
        }else{
            *ptr = (res + 4) as *const u8
        }
    }
}

fn skip_bytes_and_align4(ptr: *mut *const BigEndian32, len: usize){
    unsafe{
        let mut ptr1 = *ptr as *const u8;
        ptr1 = ptr1.add(len);
        align4(&mut ptr1);
        *ptr = ptr1 as *const BigEndian32
    }
}

fn skip_white(ptr: *mut *const BigEndian32){
    while peek(ptr) == 0 || peek(ptr) == FdtNodeType::FdtNop as u32{
        read(ptr);
    }
}

fn read_str(ptr: *mut *const BigEndian32) -> &'static str{
    unsafe{
        let mut ptr1 = *ptr as *const u8;
        let mut len = 0;
        while *ptr1 != 0{
            ptr1 = ptr1.add(1);
            len += 1;
        }
        align4(&mut ptr1);
        let res = str::from_utf8_unchecked(slice::from_raw_parts(*ptr as *const u8,len));
        *ptr = ptr1 as *const BigEndian32;
        res
    }
}

fn skip_props(ptr: *mut *const BigEndian32)->usize{
    let mut cnt = 0;
    while peek(ptr) == FdtNodeType::FdtProp as u32{
        read(ptr);
        let len = read(ptr) as usize;
        let _nameoff = read(ptr);
        skip_bytes_and_align4(ptr, len);
        cnt += 1
    }
    cnt
}

/// Enumerate all the subnodes and call the handler for every subnode
pub fn enumerate_subnodes(node: &NodeInfo, handler: fn(name: &str, node: &NodeInfo)){
    let mut ptr = node.first_child_ptr;
    for _ in 0..node.children_cnt {
        unsafe{
            handler((*ptr).name,&(*ptr));
            ptr = (*ptr).next_node as *const NodeInfo;
        }
    }
}

/// Read a node and its subnodes, and store the structs to `node_pool` memory area, and return the pointer to the new memory area
/// that has not been taken.
/// 
/// If a structure area exceeds the boundary, this function will return an error.
pub fn read_node(ptr: *mut *const BigEndian32, boundary: *const BigEndian32, node_pool: *mut NodeInfo) -> Result<*mut NodeInfo,&'static str>{
    unsafe{
        let mut res = node_pool.add(1);
        let p0 = *ptr;
        skip_white(ptr);
        if peek(ptr) == FdtNodeType::FdtEnd as u32{
            *ptr = p0;
            return Err("Bad node format: The FDT ends here.");
        }
        if read(ptr) != FdtNodeType::FdtBeginNode as u32{
            *ptr = p0;
            return Err("Bad node format: unknown node type.");
        }
        let node_name = read_str(ptr);
        skip_white(ptr);
        let prop_ptr = *ptr;
        let props_cnt = skip_props(ptr);
        skip_white(ptr);
        let mut children_cnt = 0;
        while peek(ptr) == FdtNodeType::FdtBeginNode as u32{
            res = read_node(ptr, boundary, res)?;
            skip_white(ptr);
            children_cnt += 1;
        }
        if read(ptr) != FdtNodeType::FdtEndNode as u32{
            *ptr = p0;
            return Err("Bad node format: unknown node type.");
        }
        *node_pool = NodeInfo 
        {
            name: node_name, 
            first_prop_ptr: prop_ptr, 
            props_cnt: props_cnt, 
            first_child_ptr: node_pool.add(1), 
            children_cnt: children_cnt, 
            next_node: res
        };
        Ok(res)
    }
}