//! Memory management for risc-v architecture

pub mod paging;
pub mod sv;

mod types;
pub use types::*;

use crate::{arch::MAX_HARTS, mm::stack::RawKernelStack};

/// The kernel stack
#[unsafe(link_section = ".bss.stack")]
pub static KERNEL_STACK: [RawKernelStack; MAX_HARTS] = [RawKernelStack::new(); MAX_HARTS];
