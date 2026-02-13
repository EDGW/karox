//! Device subsystem: device model, drivers and initialization.
//!
//! This crate subtree provides:
//! - types representing physical devices ([Device], [DeviceInfo]) and their resources.
//! - driver discovery and binding infrastructure located in [driver].
//! - runtime device tree root and initialization sequence invoked by [init].
//!
//! Design notes:
//! - Device nodes are represented as owned [handle::Handle]s; parent relationships use weak [handle::HandleRef]s.
//! - Drivers are discovered by compatible strings and bound via probe routines; **successful probe
//!   prevents descending into a device's children** during initialization.
//! - Keep the module API minimal and focused on device topology and orchestration.
pub mod arch;
pub mod driver;
pub mod handle;
pub mod info;
pub mod intc;
pub mod io;
pub mod mmio;
pub mod serial;

mod dev;
pub use dev::*;
mod mem;
pub use mem::*;
mod hart;
pub use hart::*;

pub fn init() {
    driver::init();
    init_devs();
}
