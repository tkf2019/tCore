use alloc::{string::String, sync::Arc, vec::Vec};
use core::cell::SyncUnsafeCell;
use id_alloc::{IDAllocator, RecycleAllocator};
use kernel_sync::{CPUs, SpinLock};
use log::{debug, error, trace, warn};
use oscomp::{fetch_test, finish_test};
use spin::Lazy;

use crate::{
    arch::{
        get_cpu_id,
        mm::{PTEFlags, PAGE_SIZE},
    },
    config::{
        ADDR_ALIGN, CPU_NUM, INIT_TASK_PATH, IS_TEST_ENV, KERNEL_STACK_PAGES, KERNEL_STACK_SIZE,
        MAIN_TASK, ROOT_DIR, TRAMPOLINE_VA,
    },
    error::KernelResult,
    loader::from_args,
    mm::{pma::FixedPMA, KERNEL_MM},
    tests,
};

use super::{
    context::{TaskContext, __switch},
    schedule::{QueueScheduler, Scheduler},
    task::Task,
    TaskState,
};

/// Global process identification allocator.
static PID_ALLOCATOR: Lazy<SpinLock<RecycleAllocator>> =
    Lazy::new(|| SpinLock::new(RecycleAllocator::new(0)));

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
static KSTACK_ALLOCATOR: Lazy<SpinLock<RecycleAllocator>> =
    Lazy::new(|| SpinLock::new(RecycleAllocator::new(0)));

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
        Arc::new(SpinLock::new(FixedPMA::new(KERNEL_STACK_PAGES)?)),
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
pub fn idle() -> ! {
    loop {
        let mut task_manager = TASK_MANAGER.lock();

        if let Some(task) = task_manager.fetch() {
            let cpu_ctx = cpu();
            let idle_ctx = &cpu_ctx.idle_ctx as *const TaskContext;
            let next_ctx = {
                let mut locked_inner = task.locked_inner();
                locked_inner.state = TaskState::Running;
                &task.inner().ctx as *const TaskContext
            };
            // Ownership moved to `current`.
            cpu_ctx.current = Some(task);

            // Release the lock.
            drop(task_manager);

            unsafe { __switch(idle_ctx, next_ctx) };
        }
    }
}

/// Current task exits. Run next task.
pub fn do_exit(exit_code: i32) {
    let curr = curr_task().unwrap();
    trace!("{:#?} exited with code {}", curr, exit_code);
    let curr_ctx = {
        let mut locked_inner = curr.locked_inner();
        curr.inner().exit_code = exit_code;
        locked_inner.state = TaskState::Zombie;
        &curr.inner().ctx as *const TaskContext
    };

    if !IS_TEST_ENV && curr.pid.0 == 0 {
        panic!("All task exited!");
    } else {
        handle_zombie(curr);
    }

    unsafe { __switch(curr_ctx, idle_ctx()) };
}

/// Current task suspends. Run next task.
pub fn do_yield() {
    let curr = curr_task().take().unwrap();
    trace!("{:#?} suspended", curr);
    let curr_ctx = {
        let mut locked_inner = curr.locked_inner();
        locked_inner.state = TaskState::Runnable;
        &curr.inner().ctx as *const TaskContext
    };

    // push back to scheduler
    TASK_MANAGER.lock().add(curr);

    unsafe {
        // Saves and restores CPU local variable, intena.
        let intena = CPUs[get_cpu_id()].intena;
        __switch(curr_ctx, idle_ctx());
        CPUs[get_cpu_id()].intena = intena;
    };
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
/// let mut child_inner = child.locked_inner();
/// let mut init_task_inner = INIT_TASK.locked_inner();
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
    let mut locked_inner = task.locked_inner();
    for child in locked_inner.children.iter() {
        if !IS_TEST_ENV {
            let mut child_inner = child.locked_inner();
            let mut init_task_inner = INIT_TASK.locked_inner();
            child_inner.parent = Some(Arc::downgrade(&INIT_TASK));
            init_task_inner.children.push(child.clone());
        }
    }
    locked_inner.children.clear();
    locked_inner.state = TaskState::Zombie;
    if IS_TEST_ENV && task.tid == MAIN_TASK {
        finish_test(task.inner().exit_code, &task.name);
    }
}
