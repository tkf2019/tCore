use alloc::{collections::VecDeque, sync::Arc};

use super::task::Task;

/// Possible interfaces for task schedulers.
pub trait Scheduler {
    /// Add a task to be scheduled sooner or later.
    fn add(&mut self, task: Arc<Task>);

    /// Get a task to run on the target processor.
    fn fetch(&mut self) -> Option<Arc<Task>>;
}

pub struct QueueScheduler {
    queue: VecDeque<Arc<Task>>,
}

impl QueueScheduler {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl Scheduler for QueueScheduler {
    fn add(&mut self, task: Arc<Task>) {
        self.queue.push_back(task);
    }

    fn fetch(&mut self) -> Option<Arc<Task>> {
        self.queue.pop_front()
    }
}
