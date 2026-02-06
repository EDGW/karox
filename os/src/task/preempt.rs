use crate::{
    arch::trap::intr::{disable_intr, restore_intr},
    dev::get_current_hart,
    task::scheduler::schedule,
};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Atomic counter indicating whether current hart is preemptable.
/// It's a member in [super::hart::HartInfo].
#[derive(Debug)]
pub struct PreemptCounter {
    counter: AtomicUsize,
    need_resched: AtomicBool,
}

impl PreemptCounter {
    pub const fn new() -> PreemptCounter {
        PreemptCounter {
            counter: AtomicUsize::new(0),
            need_resched: AtomicBool::new(false),
        }
    }

    pub fn is_preempt_allowed(&self) -> bool {
        self.counter.load(Ordering::Relaxed) == 0
    }

    /// Call [super::scheduler::schedule_manually()] after the counter is cleared.
    pub fn need_reschedule(&self) {
        self.need_resched.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }

    /// ## Notes:
    /// Since [PreemptCounter] is only expected to appear as a member inside [super::hart::HartInfo],
    /// there is no need to consider genuine asynchronous concurrent access from different harts;  
    /// the use of [AtomicUsize] and similar atomic structures is to
    /// ensure view consistency in the presence of preemption occurring during the execution of
    /// disable/restore preemption operations.
    ///
    /// Although preemption is restored, as long as [PreemptCounter::need_resched] is true,
    /// it indicates that the context is still in the Task Execution Environment, so nested scheduling cannot occur.
    ///
    /// (because we assume that [disable_preempt] and [restore_preempt] always appear in matching pairs,
    /// and the Scheduling Environment itself cannot be preempted).
    /// If [PreemptCounter::need_resched] is true, it means the counter was already nonzero when preemption occurred,
    /// so when the counter is zero, we must have returned to the Task Execution Environment.
    pub fn restore(&self) {
        let mut need_resched = false;
        if self.get_count() == 1 {
            if let Ok(_) = self.need_resched.compare_exchange(
                true,
                false,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                need_resched = true;
            }
        }
        if need_resched {
            let token = disable_intr();
            self.counter.fetch_sub(1, Ordering::Relaxed);
            schedule();
            restore_intr(token);
        } else {
            self.counter.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub fn get_count(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}

pub fn disable_preempt() {
    let hart = get_current_hart();
    hart.preempt.disable();
}

/// Restore Preemption
pub fn restore_preempt() {
    let hart = get_current_hart();
    hart.preempt.restore();
}
