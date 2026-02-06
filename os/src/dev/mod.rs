//! This module contains devices and management methods for physical devices

pub mod ic;
pub mod info;
pub mod io;
pub mod mmio;
pub mod serial;

mod devices;
pub use devices::*;
mod mem;
pub use mem::*;
mod hart;
pub use hart::*;
