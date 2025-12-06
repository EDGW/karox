//! Memory management for risc-v architecture

use crate::arch::mm::config::{KERNEL_STACK_SIZE, MAX_HARTS};
pub mod config;
pub mod paging;

/// The kernel stack
#[unsafe(link_section = ".bss.stack")]
pub static KERNEL_STACK: [[u8; KERNEL_STACK_SIZE]; MAX_HARTS] = [[0; KERNEL_STACK_SIZE]; MAX_HARTS];
