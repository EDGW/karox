use alloc::vec::Vec;
use utils::vec::LockedVecStatic;

use crate::{
    arch::{MAX_HARTS, hart::get_current_hart_id},
    task::preempt::PreemptCounter,
};

pub struct HartInfo {
    pub hart_id: usize,
    pub preempt: PreemptCounter,
}

impl HartInfo {
    pub const fn new(hart_id: usize) -> HartInfo {
        HartInfo {
            hart_id,
            preempt: PreemptCounter::new(),
        }
    }
}

static WORKING_HARTS: LockedVecStatic<&'static HartInfo> = LockedVecStatic::new();

static HARTS: [HartInfo; MAX_HARTS] = {
    const NONE: HartInfo = HartInfo::new(0);
    let mut res = [NONE; MAX_HARTS];
    let mut i = 0;
    while i < MAX_HARTS {
        res[i].hart_id = i;
        i += 1;
    }
    res
};

pub fn register_hart(hart_id: usize) {
    if hart_id > MAX_HARTS {
        log::warn!(
            "Unsupported hart id: #{:}, exceeding max hart count '{:}'.",
            hart_id,
            MAX_HARTS
        );
    }
    WORKING_HARTS.push(&HARTS[hart_id]);
}

pub fn get_working_harts() -> Vec<&'static &'static HartInfo> {
    WORKING_HARTS.clone()
}

pub fn get_current_hart() -> &'static HartInfo {
    &HARTS[get_current_hart_id()]
}
