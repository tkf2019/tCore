use alloc::collections::BTreeMap;
use spin::{mutex::Mutex, Lazy};
use talloc::RecycleAllocator;

use crate::println;

use super::{schedule::QueueScheduler, task::TASK};

/// Handlers packed for global task managers
pub struct TaskManager {
    /// PID allocator
    pub pid_allocator: RecycleAllocator,

    /// Kernel stack allocator
    pub kid_allocator: RecycleAllocator,

    /// PID is mapped to Task in this table.
    /// Speed up locating the task by PID to fetch or modify the task data.
    pub task_table: BTreeMap<usize, TASK>,

    /// Task scheduler
    pub sched: QueueScheduler,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            pid_allocator: RecycleAllocator::new(),
            kid_allocator: RecycleAllocator::new(),
            sched: QueueScheduler::new(),
            task_table: BTreeMap::new(),
        }
    }
}

pub static TASK_MANAGER: Lazy<TaskManager> = Lazy::new(|| {
    #[cfg(feature = "global_test")]
    println!("GLOBAL_TEST");

    #[cfg(feature = "local_test")]
    println!("LOCAL_TEST");
    TaskManager::new()
});
