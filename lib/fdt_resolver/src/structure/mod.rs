//! Module: FDT structure-block parser
//!
//! This module provides low-level helpers to parse the **structure block** of a Flattened Device Tree (FDT).
//! It reads big-endian 32-bit tokens and builds a flat [NodeInfo] pool that describes node names,
//! property pointers/counts, child links and sibling links.
//!
//! ## Responsibilities
//! - Walk the structure block tokens.
//! - Extract node names (zero-terminated strings) and property payload pointers (raw bytes).  
//! - Populate `NodeInfo` entries into a caller-provided pool and link children/siblings.  
//! - Provide small helpers to iterate child nodes and properties ([enumerate_subnodes], [enumerate_props]).
//!
//! ## Safety & assumptions
//! - The parser operates on raw pointers into the FDT memory — callers must ensure the `ptr`, `boundary`
//!   and any returned pointers remain valid for the required lifetime.
//! - All tokens and lengths are big-endian 32-bit values and 4-byte aligned; the code relies on that alignment.
//! - The module does not allocate; the caller must provide a `node_pool` large enough to hold all [NodeInfo] entries.
//! - Parsing functions return `Err(&'static str)` on malformed input and attempt to restore the cursor to its
//!   original value on error where applicable.
//!
//! ## Main API
//! - [read_node]
//!   Parse a node (and its subtree) starting at `*ptr`, populate `node_pool` with a [NodeInfo] and its descendants,
//!   and return the next free entry pointer on success. `fdt` is the [FdtPtr] for pointer helpers (e.g. string table).
//! - [enumerate_subnodes]
//!   Call `handler(name, node)` for each direct child of `node` in order.
//! - [enumerate_props]
//!   Iterate properties of `node`, calling `handler` with a [PropertyInfo] for each property.


use core::{slice::{self}, str};

use config::mm::endian::{BigEndian32};

use crate::FdtPtr;

pub mod memory;


/// *The structure block is composed of a sequence of pieces, each beginning with a token,*
/// *that is, a big-endian 32-bit integer. Some tokens are followed by extra data,*
/// *the format of which is determined by the token value. All tokens shall be aligned on a 32-bit boundary,*
/// *which may require padding bytes (with a value of 0x0) to be inserted after the previous token’s data.*
/// 
/// This enum defines the basic token types used in the structure block.
#[repr(u32)]
pub enum FdtNodeType{
    /// Marks the start of a node block. Followed by a NUL-terminated node name string.
    FdtBeginNode=   0x1,
    /// Marks the end of a node block.
    FdtEndNode  =   0x2,
    /// Marks a property entry. Followed by: length (u32), nameoff (u32), and `length` bytes of value (padded to 4 bytes).
    FdtProp     =   0x3,
    /// No-op token; should be ignored by parsers.
    FdtNop      =   0x4,
    /// Marks the end of the structure block.
    FdtEnd      =   0x9
}

/// Represents a parsed node stored in the provided pool.
///
/// [NodeInfo] contains pointers into the original FDT memory for names and property data,
/// plus links to children and siblings stored as [NodeInfo] entries in the pool.
///
/// The added `fdt` field gives access to FDT-level helpers (for example string-table offsets).
pub struct NodeInfo{
    /// [FdtPtr] handle for the source DTB this node references.
    pub fdt: FdtPtr,
    /// Node name string slice pointing into original FDT memory (NUL-terminated in source).
    pub name: &'static str,
    /// Pointer to the first property token for this node
    pub first_prop_ptr: *const BigEndian32,
    /// Number of properties for this node.
    pub props_cnt: usize,
    /// Pointer to the first child node's[`NodeInfo] entry in the pool.
    pub first_child_ptr: *const NodeInfo,
    /// Number of direct children.
    pub children_cnt: usize,
    /// Pointer to the next sibling node's [NodeInfo] entry in the pool (or undefined if none).
    pub next_node: *const NodeInfo,
}

impl NodeInfo{
    /// Get the basic name of the node name
    /// 
    /// Valid node name formats:
    ///  - `basic name`
    ///  - `basic name`@`starting point`
    pub fn get_basic_name(&self) -> &'static str{
        if self.name.contains('@'){
            self.name.split_once('@').unwrap().0
        }
        else{
            self.name
        }
    }

    /// Get a property with the speficied name, or return an error if not found or encountered with malformed nodes
    pub fn get_prop(&self, name: &str) -> Result<PropertyInfo,&'static str>{
        get_prop(self, name)
    }
}

/// A lightweight description of a property (name and value slices).
///
/// Both [PropertyInfo::name] and [PropertyInfo::value] borrow from the original DTB memory; the backing DTB must remain valid
/// for the lifetime of the [PropertyInfo] usage.
#[derive(Clone, Copy)]
pub struct PropertyInfo{
    /// Property name (points into the DTB string table).
    pub name: &'static str,
    /// Property value as a byte slice.
    pub value: &'static [u8]
}

impl PropertyInfo{
    /// Convert the value to a string
    /// 
    /// **This method does not guarantee that the returned string is valid**
    pub unsafe fn value_as_str(&self) -> &'static str{
        unsafe{
            str::from_utf8_unchecked(&self.value[0..self.value.len()-1]) // ignoring terminating charater
        }
    }
}

/// Read the 32-bit big-endian value at `**ptr` without advancing the cursor.
fn peek(ptr: *mut *const BigEndian32) -> u32{
    unsafe{
        (**ptr).value()
    }
}


/// Read the 32-bit big-endian value at `**ptr` and advance the cursor by 1
fn read(ptr: *mut *const BigEndian32) -> u32{
    unsafe{
        let res = (**ptr).value();
        *ptr = (*ptr).add(1);
        res
    }
}

/// Align a raw byte pointer up to the next 4-byte boundary (in-place).
///
/// This function updates the pointer to the nearest address that is 4-byte aligned.
/// It does not return anything; the caller-provided pointer is modified.
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

/// Advance the `ptr` by `len` bytes (from current byte position) and align to 4 bytes.
fn skip_bytes_and_align4(ptr: *mut *const BigEndian32, len: usize){
    unsafe{
        let mut ptr1 = *ptr as *const u8;
        ptr1 = ptr1.add(len);
        align4(&mut ptr1);
        *ptr = ptr1 as *const BigEndian32
    }
}

/// Skip padding tokens (0) and `FdtNop` tokens.
///
/// Leaves the cursor at the next non-padding, non-NOP token.
fn skip_white(ptr: *mut *const BigEndian32){
    while peek(ptr) == 0 || peek(ptr) == FdtNodeType::FdtNop as u32{
        read(ptr);
    }
}

/// Read a byte slice starting at the current byte position of `*ptr` in specific size.
///
/// Returns a `&'static str` that borrows from the original FDT memory and advances `*ptr` to the
/// next 4-byte aligned word after the termination.
fn read_bytes_by_length(ptr: *mut *const BigEndian32, len: usize) -> &'static [u8]{
    unsafe{
        let mut ptr1 = *ptr as *const u8;
        ptr1 = ptr1.add(len);
        align4(&mut ptr1);
        let res = slice::from_raw_parts(*ptr as *const u8,len);
        *ptr = ptr1 as *const BigEndian32;
        res
    }
}

/// Read a NUL-terminated string starting at the current byte position of `*ptr`.
///
/// Returns a `&'static str` that borrows from the original FDT memory and advances `*ptr` to the
/// next 4-byte aligned word after the terminating NUL.
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

/// Skip all consecutive `FdtProp` entries at the current cursor and return the count skipped.
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

/// Enumerate direct child nodes of `node`, calling `handler` for each child.
///
/// `handler` receives the child's `name` and a reference to the child's `NodeInfo`.
/// This is a safe wrapper around raw pointer traversal; the call itself uses `unsafe` internally.
pub fn enumerate_subnodes<C: FnMut(&str, &NodeInfo)>(node: &NodeInfo, mut handler: C ){
    let mut ptr = node.first_child_ptr;
    for _ in 0..node.children_cnt {
        unsafe{
            handler((*ptr).name,&(*ptr));
            ptr = (*ptr).next_node as *const NodeInfo;
        }
    }
}

/// Enumerate properties of `node`, calling `handler` with each [PropertyInfo].
/// 
/// The value returned by `handler` determines whether the iteration should continue.
///
/// - On success returns `Ok(())`.
/// - Returns `Err(&str)` if the structure is malformed while iterating properties.
///
/// This function uses `node.fdt` to obtain the string table start pointer (via [FdtPtr::get_str_table_start]).
/// Each property yields a [PropertyInfo] borrowing the DTB memory.
pub fn enumerate_props<C: FnMut(&PropertyInfo)->bool>(node: &NodeInfo, mut handler: C) -> Result<(),&'static str>{
    let mut ptr = node.first_prop_ptr;
    let st_ptr = node.fdt.get_str_table_start() as *const u8;
    for _ in 0..node.props_cnt {
        unsafe{
            skip_white(&mut ptr);
            if read(&mut ptr) != FdtNodeType::FdtProp as u32{
                return Err("Bad node format: unknown node type");
            }
            let len = read(&mut ptr) as usize;
            let nameoff = read(&mut ptr) as usize;
            let mut ptr_str = st_ptr.add(nameoff) as *const BigEndian32;
            let name = read_str(&mut ptr_str);
            let value = read_bytes_by_length(&mut ptr, len);
            let p:PropertyInfo = PropertyInfo { name, value };
            if !handler(&p){
                break;
            }
        }
    }
    Ok(())
}

/// Find a property with the given name of `node`, returning a [PropertyInfo].
///
/// - On success returns `Ok(())`.
/// - Returns `Err(&str)` if the structure is malformed while iterating properties, or the property with such a name does not exist.
///
/// This function uses `node.fdt` to obtain the string table start pointer (via [FdtPtr::get_str_table_start]).
pub fn get_prop(node: &NodeInfo, name:&str) -> Result<PropertyInfo,&'static str>{
    let mut res: Option<PropertyInfo> = None;
    enumerate_props(node, |propinfo|{
        if propinfo.name == name{
            res = Some(propinfo.clone());
            false
        }
        else{
            true
        }
    })?;
    res.ok_or("Property Info with the given name does not exist.")
}

/// Parse a node and its subtree from the FDT structure block into `node_pool`.
///
/// - `ptr`: pointer to a cursor; it will be advanced during parsing.
/// - `boundary`: pointer to the end of the structure block (caller-provided; used for bounds checks if desired).
/// - `node_pool`: pointer to the first free `NodeInfo` entry where parsed data will be stored.
///
/// On success returns `Ok(next_free)`, where `next_free` is the next unused `NodeInfo` slot after the newly
/// written entries. On failure returns `Err(&'static str)` and restores `*ptr` to its value on entry.
///
/// # Errors
/// - Returns an error for malformed node tokens (unexpected token types or early end).
///
/// # Safety
/// - The function dereferences raw pointers and assumes the FDT memory and `node_pool` are valid and writable.
/// - Caller must ensure `node_pool` has enough capacity for this node and all descendants.
/// - When encountered with errors, the pointer will be restored to its original state.
pub fn read_node(fdt:FdtPtr, ptr: *mut *const BigEndian32, boundary: *const BigEndian32, node_pool: *mut NodeInfo) -> Result<*mut NodeInfo,&'static str>{
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
            res = read_node(fdt, ptr, boundary, res)?;
            skip_white(ptr);
            children_cnt += 1;
        }
        if read(ptr) != FdtNodeType::FdtEndNode as u32{
            *ptr = p0;
            return Err("Bad node format: unknown node type.");
        }
        *node_pool = NodeInfo 
        {
            fdt,
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