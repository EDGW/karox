use core::mem::swap;

use crate::{
    sched::{Scheduler, idle::IDLE_TASKS},
    task::task::Task,
};
use alloc::{collections::vec_deque::VecDeque, sync::Arc};

pub struct FifoScheduler {
    hart_id: usize,
    running: Option<Arc<Task>>,
    queue: VecDeque<Arc<Task>>,
}
impl FifoScheduler {
    pub fn new(hart_id: usize) -> FifoScheduler {
        FifoScheduler {
            hart_id,
            running: None,
            queue: VecDeque::new(),
        }
    }
}
impl Scheduler for FifoScheduler {
    fn add_to_ready(&mut self, task: Arc<Task>) {
        self.queue.push_back(task);
    }

    fn fetch_new(&mut self) -> Arc<Task> {
        // Add
        let mut last_running: Option<Arc<Task>> = None;
        swap(&mut last_running, &mut self.running);
        if let Some(task) = last_running {
            self.add_to_ready(task);
        }
        // Fetch
        let res = self.queue.pop_front();
        self.running = res.clone();
        match res {
            Some(task) => task,
            None => IDLE_TASKS[self.hart_id].clone(),
        }
    }
}
