use alloc::{
    collections::{vec_deque, VecDeque},
    string::String,
    sync::Arc,
};
use log::warn;
use oscomp::fetch_test;

use crate::{
    config::{IS_TEST_ENV, ROOT_DIR},
    loader::from_args,
};

use super::{Task, TaskState};

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

    /// Returns a front-to-back iterator that returns immutable references.
    pub fn iter(&self) -> vec_deque::Iter<Arc<Task>> {
        self.queue.iter()
    }
}

impl Scheduler for QueueScheduler {
    fn add(&mut self, task: Arc<Task>) {
        self.queue.push_back(task);
    }

    fn fetch(&mut self) -> Option<Arc<Task>> {
        if self.queue.is_empty() && IS_TEST_ENV {
            if let Some(args) = fetch_test() {
                return from_args(String::from(ROOT_DIR), args)
                    .map_err(|err| warn!("{:?}", err))
                    .ok();
            }
            None
        } else {
            let task = self.queue.pop_front().unwrap();

            // State cannot be set to other states except [`TaskState::Runnable`] by other harts,
            // e.g. this task is waken up by another task that releases the resources.
            if task.locked_inner().state != TaskState::Runnable {
                self.queue.push_back(task);
                None
            } else {
                Some(task)
            }
        }
    }
}
