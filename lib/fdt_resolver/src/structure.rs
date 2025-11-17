//! Module: FDT structure-block parser
//!
//! This module provides low-level helpers to parse the **structure block** of a Flattened Device Tree (FDT).
//! It reads big-endian 32-bit tokens and builds a flat `NodeInfo` pool that
//! describes node names, property pointers/counts, child links and sibling links.
//!
//! ## Responsibilities
//! - Walk the structure block tokens (`FDT_BEGIN_NODE`, `FDT_PROP`, `FDT_END_NODE`, etc.).  
//! - Extract node names (zero-terminated strings) and property payload pointers (raw bytes).  
//! - Populate `NodeInfo` entries into a caller-provided pool and link children/siblings.  
//! - Provide a tiny visitor helper `enumerate_subnodes` for iteration.
//!
//! ## Safety & assumptions
//! - The parser operates entirely on raw pointers into the FDT memory — **all functions are unsafe by nature**.
//!   Callers must ensure the `ptr` and `boundary` arguments actually point into a valid, readable FDT memory region.
//! - All tokens and lengths are expected to be **big-endian 32-bit** values and 4-byte aligned; the code
//!   relies on that alignment and on well-formed token/length fields.
//! - The module **does not allocate**; the caller must provide a `node_pool` large enough to hold all `NodeInfo` entries.
//! - Out-of-format or out-of-bounds data results in an `Err(&'static str)` from `read_node` — the original `ptr` is restored on error.
//!
//! ## Main API
//! - `read_node(ptr, boundary, node_pool) -> Result<*mut NodeInfo, &'static str>`  
//!   Parse a node (and its subtree) starting at `*ptr`, populate `node_pool` with a `NodeInfo` and its descendants,
//!   and return the next free entry pointer on success. `boundary` is provided so callers may check bounds if desired.
//! - `enumerate_subnodes(node, handler)`  
//!   Call `handler(name, node)` for each direct child of `node` in order.
//!
//! ## Notes on usage
//! - Typical flow:
//!   1. Map or obtain a pointer to the FDT structure block (big-endian words).  
//!   2. Provide a `node_pool` buffer (array of `NodeInfo`) large enough for all nodes.  
//!   3. Call `read_node` with a pointer to the first token; on success, the pool contains the parsed tree.
//!   4. Use `enumerate_subnodes` to traverse child nodes or inspect `NodeInfo` fields directly.
//! - The module stores pointers into the original FDT memory for names and property data. The backing FDT memory
//!   must remain valid for the lifetime of the `NodeInfo` data.
//!
//! ## Limitations
//! - The parser is low-level and assumes a well-formed FDT. It performs limited validation and does not guard
//!   against every possible malformed input; callers in hostile environments must validate bounds/lengths before calling.
//!

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

/// This struct represents a node in the momory
pub struct NodeInfo{
    /// Node name string slice pointing into original FDT memory (NUL-terminated in source).
    pub name: &'static str,
    /// Pointer to the first property token for this node (points to the `BigEndian32` token stream).
    pub first_prop_ptr: *const BigEndian32,
    /// Number of properties for this node.
    pub props_cnt: usize,
    /// Pointer to the first child node's `NodeInfo` entry in the pool.
    pub first_child_ptr: *const NodeInfo,
    /// Number of direct children.
    pub children_cnt: usize,
    /// Pointer to the next sibling node's `NodeInfo` entry in the pool (or undefined if none).
    pub next_node: *const NodeInfo,
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
pub fn enumerate_subnodes(node: &NodeInfo, handler: fn(name: &str, node: &NodeInfo)){
    let mut ptr = node.first_child_ptr;
    for _ in 0..node.children_cnt {
        unsafe{
            handler((*ptr).name,&(*ptr));
            ptr = (*ptr).next_node as *const NodeInfo;
        }
    }
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