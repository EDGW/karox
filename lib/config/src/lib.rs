//! Configurations for the kernel.
//! The module is divided as a library for better organization and independence.

#![no_std]
#![no_main]
#![deny(missing_docs)]
#![deny(warnings)]

pub mod build_flags;

#[macro_use]
pub mod arch;
#[macro_use]
pub mod mm;