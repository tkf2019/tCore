use alloc::{collections::BTreeMap, sync::Arc, vec, vec::Vec};
use core::cell::RefCell;
use lazy_static::lazy_static;
use spin::mutex::Mutex;
use talloc::RecycleAllocator;

use crate::println;

use super::schedule::{QueueScheduler, Scheduler};
use super::task::Task;

/// Handlers packed for global task managers
pub struct TaskManager {
    /// PID allocator
    pub pid_allocator: RefCell<RecycleAllocator>,

    /// Task scheduler
    pub sched: QueueScheduler,

    /// PID is mapped to Task in this table.
    /// Speed up locating the task by PID to fetch or modify the task data.
    pub task_table: BTreeMap<usize, Arc<Mutex<Task>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            pid_allocator: RefCell::new(RecycleAllocator::new()),
            sched: QueueScheduler::new(),
            task_table: BTreeMap::new(),
        }
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = {
        #[cfg(feature = "global_test")]
        println!("GLOBAL_TEST");

        #[cfg(feature = "local_test")]
        println!("LOCAL_TEST");

        Mutex::new(TaskManager::new())
    };
}
