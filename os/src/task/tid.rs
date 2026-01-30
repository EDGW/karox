use alloc::{vec, vec::Vec};

use crate::mutex::NoPreemptSpinLock;

pub struct TaskIdAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl TaskIdAllocator {
    pub const fn new() -> TaskIdAllocator {
        TaskIdAllocator {
            current: 0,
            recycled: vec![],
        }
    }

    /// Allocate an tid.
    ///
    /// Use [alloc_tid] for safety.
    pub unsafe fn alloc(&mut self) -> usize {
        if let Some(tid) = self.recycled.pop() {
            tid
        } else {
            let res = self.current;
            self.current += 1;
            res
        }
    }

    /// Free a tid.
    /// **Duplicated free may lead to undefined behavior**
    pub unsafe fn free(&mut self, tid: usize) {
        self.recycled.push(tid);
    }
}

/// Safe wrapper around tids that frees the managed tid on drop.
#[derive(Debug)]
pub struct TaskId {
    inner: usize,
}

impl TaskId {
    pub fn value(&self) -> usize {
        self.inner
    }
}

impl Drop for TaskId {
    fn drop(&mut self) {
        unsafe {
            TID_ALLOC.lock().free(self.inner);
        }
    }
}

pub static TID_ALLOC: NoPreemptSpinLock<TaskIdAllocator> = NoPreemptSpinLock::new(TaskIdAllocator::new());

pub fn alloc_tid() -> TaskId {
    TaskId {
        inner: unsafe { TID_ALLOC.lock().alloc() },
    }
}
