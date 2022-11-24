use alloc::sync::Arc;
use log::trace;
use spin::{Lazy, Mutex};
use talloc::{IDAllocator, RecycleAllocator};
use tmm_rv::{PTEFlags, PAGE_SIZE};
use tvfs::{OpenFlags, VFS};

use crate::{
    config::{ADDR_ALIGN, KERNEL_STACK_PAGES, KERNEL_STACK_SIZE, TRAMPOLINE_VA},
    error::KernelResult,
    fs::DISK_FS,
    mm::{pma::FixedPMA, KERNEL_MM},
};

use super::{
    context::{TaskContext, __move_to_next, __switch},
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
    KERNEL_MM.lock().alloc_write(
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

/// Schedules the task running on each CPU.
pub struct TaskManager {
    /// Task scheduler
    pub sched: QueueScheduler,

    /// CPU contexts mapped by hartid.
    pub cpus: CPUContext,
}

impl TaskManager {
    /// Create a new task manager.
    pub fn new() -> Self {
        Self {
            sched: QueueScheduler::new(),
            // cpus: BTreeMap::new(),
            cpus: CPUContext::new(),
        }
    }
}

pub static TASK_MANAGER: Lazy<Mutex<TaskManager>> = Lazy::new(|| {
    let task_manager = TaskManager::new();
    Mutex::new(task_manager)
});

/// Get current task running on this cpu.
pub fn current_task() -> Option<Arc<Task>> {
    let task_manager = TASK_MANAGER.lock();
    let cpu_context = &task_manager.cpus;
    cpu_context.current.as_ref().map(Arc::clone)
}

pub static INIT_TASK: Lazy<Arc<Task>> = Lazy::new(|| {
    // New process identification
    let pid = pid_alloc();

    // New kernel stack for user task
    let kstack = kstack_alloc();

    // Init task
    let init_task = {
        let fs = DISK_FS.lock();
        let init_task = unsafe {
            fs.open("rcore/hello_world", OpenFlags::O_RDONLY)
                .unwrap()
                .read_all()
        };
        Arc::new(Task::new(pid, kstack, init_task.as_slice()).unwrap())
    };

    // Update task manager
    let mut task_manager = TASK_MANAGER.lock();
    task_manager.cpus.current = Some(init_task.clone());
    task_manager.sched.add(init_task.clone());

    init_task
});

/// Initialize  [`INIT_TASK`] manually.
pub fn init() {
    #[allow(unused_must_use)]
    INIT_TASK.clone();
}

/// IDLE task:
/// 1.  Each cpu tries to acquire the lock.
/// 2.  Each cpu runs the task fetched from schedule queue
pub fn idle() -> ! {
    loop {
        if let Some(mut task_manager) = TASK_MANAGER.try_lock() {
            if let Some(task) = task_manager.sched.fetch() {
                let idle_ctx = &task_manager.cpus.idle_ctx as *const TaskContext;
                let next_ctx = {
                    let mut task_inner = task.inner_lock();
                    task_inner.state = TaskState::Running;
                    &task_inner.ctx as *const TaskContext
                };
                task_manager.cpus.current = Some(task);
                drop(task_manager);
                unsafe {
                    __switch(idle_ctx, next_ctx);
                }
            } else {
                panic!("No task to execute!");
            }
        }
    }
}

/// Exit current task and run next task.
pub fn do_exit(exit_code: i32) {
    let current = current_task().unwrap();
    trace!("Task {} exited with code {}", current.tid, exit_code);

    let mut current_inner = current.inner_lock();
    current_inner.exit_code = exit_code;
    current_inner.state = TaskState::Zombie;
    drop(current_inner);
    drop(current);

    let mut task_manager = TASK_MANAGER.lock();
    let idle_ctx = &mut task_manager.cpus.idle_ctx as *const TaskContext;
    drop(task_manager);
    unsafe {
        __move_to_next(idle_ctx);
    }
}
