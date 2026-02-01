use core::arch::asm;
use num_enum::FromPrimitive;

pub struct SbiTable;

pub const SBI_CON_PUTCHR: (usize, usize) = (0x01, 0);
//pub const CON_GETCHR: (usize, usize) = (0x02, 0);
pub const SBI_SET_TIMER: (usize, usize) = (0x54494D45, 0);
pub const SBI_HART_START: (usize, usize) = (0x48534D, 0);
pub const SBI_HART_STOP: (usize, usize) = (0x48534D, 1);
pub const SBI_GET_STATUS: (usize, usize) = (0x48534D, 2);
pub const SBI_SEND_IPI: (usize, usize) = (0x735049, 0);

impl SbiTable {
    pub fn console_putchr(chr: char) -> Result<(), SbiError> {
        sbi_call(SBI_CON_PUTCHR, chr as usize, 0, 0)?;
        Ok(())
    }
    pub fn set_timer(time: usize) -> Result<(), SbiError> {
        sbi_call(SBI_SET_TIMER, time, 0, 0)?;
        Ok(())
    }

    pub fn send_ipi(hart_mask: usize, hart_mask_base: usize) -> Result<(), SbiError> {
        sbi_call(SBI_SEND_IPI, hart_mask, hart_mask_base, 0)?;
        Ok(())
    }

    /// The `a1` register of the given hart will be filled with `opaque`.
    pub fn hart_start(hart_id: usize, start_addr: usize, opaque: usize) -> Result<(), SbiError> {
        sbi_call(SBI_HART_START, hart_id, start_addr, opaque)?;
        Ok(())
    }

    pub fn hart_stop() -> Result<(), SbiError> {
        sbi_call(SBI_HART_STOP, 0, 0, 0)?;
        Ok(())
    }

    pub fn hart_get_status(hart_id: usize) -> Result<HartStatus, SbiError> {
        let res = sbi_call(SBI_GET_STATUS, hart_id, 0, 0)?;
        let status = HartStatus::from_primitive(res);
        Ok(status)
    }
}

#[inline(always)]
fn sbi_call(
    eid_fid: (usize, usize),
    arg0: usize,
    arg1: usize,
    arg2: usize,
) -> Result<usize, SbiError> {
    let (eid, fid) = eid_fid;
    let mut ret_a0: isize;
    let mut ret_a1: usize;
    unsafe {
        asm!(
            // "li x16, 0",
            "ecall",
            inlateout("a0") arg0 => ret_a0,
            inlateout("a1") arg1 => ret_a1,
            in("a2") arg2,
            in("a6") fid,
            in("a7") eid,
        );
    }
    if ret_a0 != 0 {
        return Err(SbiError::from_primitive(ret_a0));
    } else {
        return Ok(ret_a1);
    }
}

#[derive(Debug, FromPrimitive, PartialEq, Eq)]
#[repr(isize)]
pub enum SbiError {
    Failed = -1,           // Failed
    NotSupported = -2,     // Not supported
    InvalidParam = -3,     // Invalid parameter(s)
    Denied = -4,           // Denied or not allowed
    InvalidAddress = -5,   // Invalid address(s)
    AlreadyAvailable = -6, // Already available
    AlreadyStarted = -7,   // Already started
    AlreadyStopped = -8,   // Already stopped
    NoShmem = -9,          // Shared memory not available
    InvalidState = -10,    // Invalid state
    BadRange = -11,        // Bad (or invalid) range
    Timeout = -12,         // Failed due to timeout
    Io = -13,              // Input/Output error
    DeniedLocked = -14,    // Denied or not allowed due to lock status
    #[default]
    Unknown = isize::MIN,
}

#[derive(Debug, FromPrimitive, PartialEq, Eq)]
#[repr(usize)]
pub enum HartStatus {
    Started = 0,
    Stopped = 1,
    StartPending = 2,
    StopPending = 3,
    Suspended = 4,
    SuspendPending = 5,
    ResumePending = 6,
    #[default]
    Unknown = 7,
}
