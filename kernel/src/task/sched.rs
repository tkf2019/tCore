use alloc::{
    collections::{vec_deque, VecDeque},
    string::String,
    sync::Arc,
    vec::Vec,
};
use core::{cell::SyncUnsafeCell, panic};
use kernel_sync::{CPUs, SpinLock};
use oscomp::fetch_test;
use spin::Lazy;

use crate::{
    arch::{get_cpu_id, TaskContext, __switch},
    config::*,
    loader::from_args,
};

use super::{Task, TaskState, handle_zombie};

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
        if IS_TEST_ENV && self.queue.is_empty() {
            if let Some(args) = fetch_test() {
                if let Some(task) = from_args(String::from(ROOT_DIR), args)
                    .map_err(|_| log::warn!("TEST NOT FOUND"))
                    .ok()
                {
                    self.queue.push_back(task);
                }
            }
        }

        if self.queue.is_empty() {
            return None;
        }

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

/// Reserved for future SMP usage.
pub struct CPUContext {
    /// Current task.
    pub curr: Option<Arc<Task>>,

    /// Idle task context.
    pub idle_ctx: TaskContext,
}

impl CPUContext {
    /// A hart joins to run tasks
    pub fn new() -> Self {
        Self {
            curr: None,
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

/// Gets current task context.
///
/// # Safety
///
/// [`TaskContext`] cannot be modified by other tasks, thus we can access it with raw pointer.
pub unsafe fn curr_ctx() -> *const TaskContext {
    &cpu().curr.as_ref().unwrap().inner().ctx
}

/// IDLE task context on this CPU.
pub fn idle_ctx() -> *const TaskContext {
    &cpu().idle_ctx as _
}

/// Kernel init task which will never be dropped.
pub static INIT_TASK: Lazy<Arc<Task>> = Lazy::new(|| Arc::new(Task::init().unwrap()));

/// Reclaim resources delegated to [`INIT_TASK`].
pub fn init_reclaim() {
    let mut init = INIT_TASK.locked_inner();
    init.children.clear();
}

/// IDLE task:
///
/// 1. Each cpu tries to acquire the lock of global task manager.
/// 2. Each cpu runs the task fetched from schedule queue.
/// 3. Handle the final state after a task finishes `do_yield` or `do_exit`.
/// 4. Reclaim resources handled by [`INIT_TASK`].
pub unsafe fn idle() -> ! {
    loop {
        init_reclaim();

        let mut task_manager = TASK_MANAGER.lock();

        if let Some(task) = task_manager.fetch() {
            let next_ctx = {
                let mut locked_inner = task.locked_inner();
                locked_inner.state = TaskState::RUNNING;
                &task.inner().ctx as *const TaskContext
            };
            log::trace!("Run {:?}", task);
            // Ownership moved to `current`.
            cpu().curr = Some(task);

            // Release the lock.
            drop(task_manager);

            __switch(idle_ctx(), next_ctx);
            
            let curr = cpu().curr.take().unwrap();
            let state = curr.get_state();
            if state == TaskState::RUNNABLE {
                TASK_MANAGER.lock().add(curr);
            } else if state == TaskState::ZOMBIE {
                handle_zombie(curr);
            } else {
                panic!("Unexpected state {:#?}", state);
            }
        }
    }
}

/// Current task suspends. Run next task.
///
/// # Safety
///
/// Unsafe context switch will be called in this function.
pub unsafe fn do_yield() {
    let curr = cpu().curr.as_ref().unwrap();
    log::trace!("{:#?} suspended", curr);
    let curr_ctx = {
        let mut locked_inner = curr.locked_inner();
        locked_inner.state = TaskState::RUNNABLE;
        &curr.inner().ctx as *const TaskContext
    };

    // Saves and restores CPU local variable, intena.
    let intena = CPUs[get_cpu_id()].intena;
    __switch(curr_ctx, idle_ctx());
    CPUs[get_cpu_id()].intena = intena;
}
