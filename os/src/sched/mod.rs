use crate::task::task::Task;
use alloc::sync::Arc;

pub mod idle;

mod fifo;
pub type DefaultScheduler = fifo::FifoScheduler;

pub trait Scheduler {
    fn add_to_ready(&mut self, task: Arc<Task>);
    fn fetch_new(&mut self) -> Arc<Task>;
}
