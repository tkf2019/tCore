use alloc::collections::VecDeque;

use super::task::TASK;

/// Possible interfaces for task schedulers.
pub trait Scheduler {
    /// Add a task to be scheduled sooner or later.
    fn add(&mut self, task: TASK);

    /// Get a task to run on the target processor.
    fn fetch(&mut self) -> Option<TASK>;
}

pub struct QueueScheduler {
    queue: VecDeque<TASK>,
}

impl QueueScheduler {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl Scheduler for QueueScheduler {
    fn add(&mut self, task: TASK) {
        self.queue.push_back(task);
    }

    fn fetch(&mut self) -> Option<TASK> {
        self.queue.pop_front()
    }
}
