use alloc::{
    collections::{vec_deque, VecDeque},
    string::String,
    sync::Arc,
    vec::Vec,
};
use core::cell::SyncUnsafeCell;
use kernel_sync::{CPUs, SpinLock};
use log::warn;
use oscomp::fetch_test;
use spin::Lazy;

use crate::{
    arch::{get_cpu_id, TaskContext, __switch},
    config::*,
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
                    .map_err(|_| warn!("TEST NOT FOUND"))
                    .ok();
            }
            None
        } else {
            let task = self.queue.pop_front().unwrap();

            // State cannot be set to other states except [`TaskState::Runnable`] by other harts,
            // e.g. this task is waken up by another task that releases the resources.
            if task.locked_inner().state != TaskState::RUNNABLE {
                self.queue.push_back(task);
                None
            } else {
                Some(task)
            }
        }
    }
}

/// Reserved for future SMP usage.
pub struct CPUContext {
    /// Current task.
    pub current: Option<Arc<Task>>,

    /// Idle task context.
    pub idle_ctx: TaskContext,
}

impl CPUContext {
    /// A hart joins to run tasks
    pub fn new() -> Self {
        Self {
            current: None,
            idle_ctx: TaskContext::zero(),
        }
    }
}

/// Global task manager shared by CPUs.
pub static TASK_MANAGER: Lazy<SpinLock<QueueScheduler>> =
    Lazy::new(|| SpinLock::new(QueueScheduler::new()));

/// Global cpu local states.
pub static CPU_LIST: Lazy<SyncUnsafeCell<Vec<CPUContext>>> = Lazy::new(|| {
    let mut cpu_list = Vec::new();
    for cpu_id in 0..CPU_NUM {
        cpu_list.push(CPUContext::new());
    }
    SyncUnsafeCell::new(cpu_list)
});

/// Returns this cpu context.
pub fn cpu() -> &'static mut CPUContext {
    unsafe { &mut (*CPU_LIST.get())[get_cpu_id()] }
}

/// Gets current task running on this CPU.
pub fn curr_task() -> Option<Arc<Task>> {
    cpu().current.as_ref().map(Arc::clone)
}

/// IDLE task context on this CPU.
pub fn idle_ctx() -> *const TaskContext {
    &cpu().idle_ctx as _
}

/// Gets current task context.
///
/// # Safety
///
/// [`TaskContext`] cannot be modified by other tasks, thus we can access it with raw pointer.
pub unsafe fn curr_ctx() -> *const TaskContext {
    &curr_task().unwrap().inner().ctx
}

pub static INIT_TASK: Lazy<Arc<Task>> = Lazy::new(|| {
    // Init task
    let args = if IS_TEST_ENV {
        fetch_test().unwrap()
    } else {
        Vec::from([String::from(INIT_TASK_PATH)])
    };
    let init_task = from_args(String::from(ROOT_DIR), args).unwrap();
    // Update task manager
    cpu().current = Some(init_task.clone());
    TASK_MANAGER.lock().add(init_task.clone());
    init_task
});

/// Initialize  [`INIT_TASK`] manually.
#[allow(unused)]
pub fn init() {
    INIT_TASK.clone();
}

/// IDLE task:
///
/// 1. Each cpu tries to acquire the lock of global task manager.
/// 2. Each cpu runs the task fetched from schedule queue.
/// 3. Handle the final state after a task finishes `do_yield` or `do_exit`.
pub unsafe fn idle() -> ! {
    loop {
        let mut task_manager = TASK_MANAGER.lock();

        if let Some(task) = task_manager.fetch() {
            let next_ctx = {
                let mut locked_inner = task.locked_inner();
                locked_inner.state = TaskState::RUNNING;
                &task.inner().ctx as *const TaskContext
            };
            log::info!("Run {:?}", task);
            // Ownership moved to `current`.
            cpu().current = Some(task);

            // Release the lock.
            drop(task_manager);

            __switch(idle_ctx(), next_ctx);
        }
    }
}

/// Current task suspends. Run next task.
///
/// # Safety
///
/// Unsafe context switch will be called in this function.
pub unsafe fn do_yield() {
    let curr = curr_task().take().unwrap();
    log::info!("{:#?} suspended", curr);
    let curr_ctx = {
        let mut locked_inner = curr.locked_inner();
        locked_inner.state = TaskState::RUNNABLE;
        &curr.inner().ctx as *const TaskContext
    };

    // push back to scheduler
    TASK_MANAGER.lock().add(curr);

    // Saves and restores CPU local variable, intena.
    let intena = CPUs[get_cpu_id()].intena;
    __switch(curr_ctx, idle_ctx());
    CPUs[get_cpu_id()].intena = intena;
}