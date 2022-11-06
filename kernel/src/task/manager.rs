use alloc::{collections::BTreeMap, sync::Arc};
use easy_fs::{FSManager, OpenFlags};
use spin::{Lazy, Mutex};
use talloc::{IDAllocator, RecycleAllocator};
use tmm_rv::{PTEFlags, PAGE_SIZE};

use crate::{
    config::{KERNEL_STACK_PAGES, KERNEL_STACK_SIZE, TRAMPOLINE_VA},
    error::KernelResult,
    fs::{read_all, FS},
    mm::{pma::FixedPMA, KERNEL_MM},
};

use super::{
    context::{switch, TaskContext},
    schedule::QueueScheduler,
    task::{Task, TASK},
};

/// Handlers packed for global task managers
pub struct TaskManager {
    /// PID allocator
    pub pid_allocator: RecycleAllocator,

    /// Kernel stack allocator
    pub kstack_allocator: RecycleAllocator,

    /// PID is mapped to Task in this table.
    /// Speed up locating the task by PID to fetch or modify the task data.
    pub task_table: BTreeMap<usize, TASK>,

    /// Task scheduler
    pub sched: QueueScheduler,

    /// Current task
    pub current: Option<TASK>,

    /// Idle task context
    pub idle_ctx: TaskContext,
}

impl TaskManager {
    /// Create a new task manager.
    pub fn new() -> Self {
        Self {
            pid_allocator: RecycleAllocator::new(),
            kstack_allocator: RecycleAllocator::new(),
            sched: QueueScheduler::new(),
            task_table: BTreeMap::new(),
            current: None,
            idle_ctx: TaskContext::zero(),
        }
    }
}

pub static TASK_MANAGER: Lazy<Mutex<TaskManager>> = Lazy::new(|| Mutex::new(TaskManager::new()));

/// Returns kernel stack layout [top, base) by kernel stack identification.
///
/// Stack grows from high address to low address.
pub fn kstack_layout(kstack: usize) -> (usize, usize) {
    let base = TRAMPOLINE_VA - kstack * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let top = base - KERNEL_STACK_SIZE;
    (top, base)
}

/// Allocate a kernel stack for the task by kernel stack identification.
///
/// Returns the kernel stack base.
pub fn kstack_alloc(kstack: usize) -> KernelResult<usize> {
    let (kstack_top, kstack_base) = kstack_layout(kstack);
    KERNEL_MM.lock().alloc_write(
        None,
        kstack_top.into(),
        kstack_base.into(),
        PTEFlags::READABLE | PTEFlags::WRITABLE,
        Arc::new(Mutex::new(FixedPMA::new(KERNEL_STACK_PAGES)?)),
    );
    Ok(kstack_base)
}

/// Get current task running on this cpu.
pub fn current_task() -> TASK {
    TASK_MANAGER.lock().current.clone().unwrap()
}

/// Add the init task into scheduler.
pub fn init() {
    let init_task = read_all(FS.open("hello_world", OpenFlags::RDONLY).unwrap());
    let mut task_manager = TASK_MANAGER.lock();

    // New process identification
    let pid = task_manager.pid_allocator.alloc();

    // New kernel stack for user task
    let kstack = task_manager.kstack_allocator.alloc();

    // Init task
    let init_task = Arc::new(Mutex::new(
        Task::new(pid, kstack, init_task.as_slice()).expect("Failed to create init task"),
    ));
    task_manager.current = Some(init_task.clone());
    task_manager.task_table.insert(pid, init_task.clone());

    let idle_ctx = &mut task_manager.idle_ctx as *mut TaskContext;
    let init_ctx = &mut init_task.lock().ctx as *mut TaskContext;
    unsafe {
        switch(idle_ctx, init_ctx);
    }
}
