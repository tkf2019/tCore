use alloc::{string::String, sync::Arc, vec::Vec};
use log::{error, trace};
use oscomp::{fetch_test, finish_test};
use spin::{Lazy, Mutex};
use talloc::{IDAllocator, RecycleAllocator};
use tmm_rv::{PTEFlags, PAGE_SIZE};

use crate::{
    config::{
        ADDR_ALIGN, CPU_NUM, INIT_TASK_PATH, IS_TEST_ENV, KERNEL_STACK_PAGES, KERNEL_STACK_SIZE,
        MAIN_TASK, ROOT_DIR, TRAMPOLINE_VA,
    },
    error::KernelResult,
    get_cpu_id,
    loader::from_args,
    mm::{pma::FixedPMA, KERNEL_MM},
};

use super::{
    context::{TaskContext, __switch},
    schedule::{QueueScheduler, Scheduler},
    task::Task,
    TaskState,
};

/// Global process identification allocator.
static PID_ALLOCATOR: Lazy<Mutex<RecycleAllocator>> =
    Lazy::new(|| Mutex::new(RecycleAllocator::new(0)));

/// Only provides [`pid_alloc()`] interface.
pub fn pid_alloc() -> usize {
    PID_ALLOCATOR.lock().alloc()
}

#[derive(Debug)]
pub struct PID(pub usize);

impl Drop for PID {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.0)
    }
}

/// Global kernal stack allocator.
static KSTACK_ALLOCATOR: Lazy<Mutex<RecycleAllocator>> =
    Lazy::new(|| Mutex::new(RecycleAllocator::new(0)));

/// Allocate new kernel stack identification.
pub fn kstack_alloc() -> usize {
    KSTACK_ALLOCATOR.lock().alloc()
}

/// Deallocate kernel stack by identification.
pub fn kstack_dealloc(kstack: usize) {
    KSTACK_ALLOCATOR.lock().dealloc(kstack)
}

/// Returns kernel stack layout [top, base) by kernel stack identification.
///
/// Stack grows from high address to low address.
pub fn kstack_layout(kstack: usize) -> (usize, usize) {
    let base = TRAMPOLINE_VA - kstack * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let top = base - KERNEL_STACK_SIZE;
    (top, base - ADDR_ALIGN)
}

/// Allocate a kernel stack for the task by kernel stack identification.
///
/// Returns the kernel stack base.
pub fn kstack_vm_alloc(kstack: usize) -> KernelResult<usize> {
    let (kstack_top, kstack_base) = kstack_layout(kstack);
    KERNEL_MM.lock().alloc_write_vma(
        None,
        kstack_top.into(),
        kstack_base.into(),
        PTEFlags::READABLE | PTEFlags::WRITABLE,
        Arc::new(Mutex::new(FixedPMA::new(KERNEL_STACK_PAGES)?)),
    )?;
    Ok(kstack_base)
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

pub struct TaskManager {
    /// Task scheduler
    pub sched: QueueScheduler,

    /// CPU contexts mapped by hartid.
    pub cpus: Vec<CPUContext>,
}

impl TaskManager {
    /// Create a new task manager.
    pub fn new() -> Self {
        Self {
            sched: QueueScheduler::new(),
            cpus: Vec::new(),
        }
    }
}

/// Global task manager shared by CPUs.
pub static TASK_MANAGER: Lazy<Mutex<TaskManager>> = Lazy::new(|| {
    let mut task_manager = TaskManager::new();
    // Initialize CPU contexts.
    for cpu_id in 0..CPU_NUM {
        task_manager.cpus.push(CPUContext::new());
    }
    Mutex::new(task_manager)
});

/// Gets current task running on this CPU.
pub fn current_task() -> Option<Arc<Task>> {
    let cpu_id = get_cpu_id();
    let task_manager = TASK_MANAGER.lock();
    let cpu_ctx = &task_manager.cpus[cpu_id];
    cpu_ctx.current.as_ref().map(Arc::clone)
}

/// IDLE task context on this CPU.
pub fn idle_ctx() -> *const TaskContext {
    let cpu_id = get_cpu_id();
    let task_manager = TASK_MANAGER.lock();
    &task_manager.cpus[cpu_id].idle_ctx as _
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
    let mut task_manager = TASK_MANAGER.lock();
    task_manager.cpus[get_cpu_id()].current = Some(init_task.clone());
    task_manager.sched.add(init_task.clone());
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
pub fn idle() -> ! {
    loop {
        let mut task_manager = TASK_MANAGER.lock();
        let mut task = task_manager.sched.fetch();
        if IS_TEST_ENV && task.is_none() {
            // Task path.
            if let Some(args) = fetch_test() {
                task = from_args(String::from(ROOT_DIR), args)
                    .map_err(|err| error!("{:?}", err))
                    .ok();
            }
        }

        if let Some(task) = task {
            let cpu_id = get_cpu_id();
            let cpu_ctx = &mut task_manager.cpus[cpu_id];
            let idle_ctx = &cpu_ctx.idle_ctx as *const TaskContext;
            let next_ctx = {
                let mut task_inner = task.inner_lock();
                task_inner.state = TaskState::Running;
                &task_inner.ctx as *const TaskContext
            };
            // Ownership moved to `current`.
            cpu_ctx.current = Some(task);
            // Release the lock.
            drop(task_manager);

            unsafe { __switch(idle_ctx, next_ctx) };

            // Back to idle task.
            let current = current_task().take().expect("From IDLE task");
            match current.get_state() {
                TaskState::Runnable => {
                    let mut task_manager = TASK_MANAGER.lock();
                    task_manager.sched.add(current);
                }
                TaskState::Zombie => {
                    if !IS_TEST_ENV && current.pid.0 == 0 {
                        panic!("All task exited!");
                    } else {
                        handle_zombie(current);
                    }
                }
                _ => {
                    panic!("Invalid task state back to idle!");
                }
            }
        }
    }
}

/// Current task exits. Run next task.
pub fn do_exit(exit_code: i32) {
    let curr_ctx = {
        let current = current_task().unwrap();
        trace!("{:#?} exited with code {}", current, exit_code);
        let mut current_inner = current.inner_lock();
        current_inner.exit_code = exit_code;
        current_inner.state = TaskState::Zombie;
        &current_inner.ctx as *const TaskContext
    };
    unsafe { __switch(curr_ctx, idle_ctx()) };
}

/// Current task suspends. Run next task.
pub fn do_yield() {
    let curr_ctx = {
        let current = current_task().unwrap();
        trace!("{:#?} suspended", current);
        let mut current_inner = current.inner_lock();
        current_inner.state = TaskState::Runnable;
        &current_inner.ctx as *const TaskContext
    };
    unsafe { __switch(curr_ctx, idle_ctx()) };
}

/// Handle zombie tasks.
/// 1. Children of current task will be delegated to [`INIT_TASK`].
/// 2. Current task may need to send a signal to its parent.
///
/// # DEAD LOCK
///
/// 1. Current task and its children are all in this function. The inner lock will be held.
/// 2. Current task acquires the lock of [`INIT_TASK`] but one of its children is waiting
/// for this lock.
/// 3. Current task cannot acquire the lock of this child, while this child cannot release
/// its lock until successfully acquiring the lock of [`INIT_TASK`].
///
/// So we need to acquire the locks in order:
///
/// ```
/// let mut child_inner = child.inner_lock();
/// let mut init_task_inner = INIT_TASK.inner_lock();
/// ```
///
/// to eliminate the dead lock when both current task and its child are trying acquire
/// the lock of [`INIT_TASK`]. If current task acquires the lock of its child, the child
/// must not be in this function, thus current task can successfully acquire the lock of
/// [`INIT_TASK`] with this child stuck at the beginning of this function. If current task
/// fails to acquire the lock of its child, it cannot acquire the lock of [`INIT_TASK`],
/// either, thus the child just in this function can acquire the lock of [`INIT_TASK`]
/// successfully and finally release both locks.
pub fn handle_zombie(task: Arc<Task>) {
    let mut inner = task.inner_lock();
    for child in inner.children.iter() {
        if !IS_TEST_ENV {
            let mut child_inner = child.inner_lock();
            let mut init_task_inner = INIT_TASK.inner_lock();
            child_inner.parent = Some(Arc::downgrade(&INIT_TASK));
            init_task_inner.children.push(child.clone());
        }
    }
    inner.children.clear();
    inner.state = TaskState::Zombie;
    if IS_TEST_ENV && task.tid == MAIN_TASK {
        finish_test(inner.exit_code, &task.name);
    }
}
