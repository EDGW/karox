//! CPU driver and per-CPU interrupt-controller binding for RISC-V device tree nodes.
//!
//! Overview and responsibilities:
//! - Implement a [Driver] that recognizes CPU device nodes by the compatible string `"riscv"`.
//! - Interpret the first MMIO range entry `dev.info.io_addr[0].start` as the CPU `hart_id`.
//! - Validate `hart_id` against [arch::MAX_HARTS] and return a probe error on overflow.
//! - Locate a child device that implements an interrupt controller:
//!   - Child must have [DeviceType::INTC], include the compatible string `"riscv,cpu-intc"`, and carry [IntcInfo].
//!   - On match, register the controller via [register_intc] and store the returned handle in the current
//!     hart's per-hart storage using [get_current_hart()].intc.call_once(...) so the handle is retained
//!     for the runtime. **This moves ownership of the registered controller from a global container into
//!     per-hart storage.**
//!
//!
//! Important notes:
//! - This module only implements the driver binding logic; the real interrupt handling is provided
//!   by the registered interrupt-controller implementation via [Intc].
use crate::{
    arch::MAX_HARTS,
    dev::{
        Device, DeviceType,
        driver::{Driver, DriverProbeError, MmioError},
        get_current_hart,
        handle::Handle,
        intc::{IntcDev, register_intc},
    },
    panic_init,
};
use alloc::boxed::Box;
use log::warn;

/// Minimal per-CPU interrupt-controller adapter.
///
/// Implement [IntcDev] as a lightweight adapter used only for registration.
/// Both methods call `unreachable!()` because this adapter does not implement
/// real interrupt handling; the actual controller implementation is expected
/// to provide runtime claim/complete semantics after registration.
pub struct CpuIntc;
impl IntcDev for CpuIntc {
    fn claim(&self) -> usize {
        unreachable!();
    }

    fn complete(&self, _ir: usize) {
        unreachable!();
    }
}

/// Driver that binds RISC-V CPU device nodes.
#[derive(Debug)]
pub struct CpuDriver;

impl CpuDriver {
    /// Create a new [CpuDriver].
    pub fn new() -> CpuDriver {
        CpuDriver
    }
}

impl Driver for CpuDriver {
    fn get_name(&self) -> &'static str {
        "RISC-V CPU"
    }

    fn get_comp_strs(&self) -> &'static [&'static str] {
        &["riscv"]
    }

    /// Probe the CPU device:
    /// - Validate MMIO presence and hart id range.
    /// - Walk children to find and register the CPU interrupt controller.
    /// - Store the registered controller handle in the current hart's storage.
    ///
    /// Error handling and semantics:
    /// - If `io_addr` is empty, probe returns [DriverProbeError::Mmio] with [MmioError::AddressNotSpecified].
    /// - If the derived `hart_id` exceeds [crate::arch::MAX_HARTS], probe returns [DriverProbeError::Mmio] with [MmioError::InvalidAddress]
    ///   and logs a warning.
    /// - If no suitable interrupt-controller child is found, call [panic_init] to abort bringup. **This is fatal and deliberate.**
    /// - If registering the controller fails, call [panic_init] to surface errors early during bringup.
    fn probe(&self, dev: Handle<Device>) -> Result<(), DriverProbeError> {
        let io_addr = &dev.info.io_addr;
        if io_addr.is_empty() {
            // No MMIO address; fail early with clear error.
            return Err(DriverProbeError::Mmio(MmioError::AddressNotSpecified));
        }

        // Interpret the first MMIO range `start` as the hart id.
        let hart_id = io_addr[0].start;
        if hart_id > MAX_HARTS {
            // Hart id out of supported range; log and return error.
            warn!("Error loading cpu: Hart id '{hart_id}' exceeding max supported value.");
            return Err(DriverProbeError::Mmio(MmioError::InvalidAddress));
        }

        // Search children for an interrupt-controller that matches the CPU intc compatible string.
        let guard = dev.children.read();
        let mut matched = false;
        for sub in &*guard {
            if !sub.info.dev_type.contains(DeviceType::INTC) {
                continue;
            }
            for comp in &sub.info.comp_list {
                // Match component and require interrupt-controller metadata.
                if comp.as_ref().eq("riscv,cpu-intc")
                    && let Some(intc_info) = &sub.info.intc_info
                {
                    let intc = CpuIntc;
                    // Register the controller and panic on failure to ensure early visibility of errors.
                    let intc =
                        register_intc(intc_info.intc_id, Box::new(intc)).unwrap_or_else(|err| {
                            panic_init!("Error registering cpu interrupt controller: {:?}", err)
                        });
                    // Store the handle to keep the controller alive.
                    get_current_hart().intc.call_once(|| intc);
                    matched = true;
                }
            }
        }

        if !matched {
            // Fail fast: CPU without its interrupt controller is fatal for bringup.
            panic_init!(
                "Error registering cpu #{}: Cpu interrupt controller not found.",
                hart_id
            );
        }
        Ok(())
    }

    fn on_registered(&self) {}
}
