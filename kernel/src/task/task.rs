use alloc::{sync::Arc, vec::Vec};
use spin::mutex::Mutex;
use talloc::{IDAllocator, RecycleAllocator};
use tmm_rv::{PTEFlags, LOW_MAX_VA, PAGE_SIZE};

use crate::{
    config::{TRAMPOLINE_VA, USER_STACK_BASE, USER_STACK_PAGES, USER_STACK_SIZE},
    error::KernelResult,
    mm::{from_elf, pma::FixedPMA, MM},
    trap::user_trap_return,
};

use super::{context::TaskContext, manager::kstack_alloc};

/// Five-state model:
///
/// - **Running** or **Runnable** (R): The task takes up a CPU core to execute its code.
/// - **Sleeping** states: **Interruptible** (S) and **Uninterruptible** (D).
/// S will only for resources to be available, while D will react to both signals and the
/// availability of resources.
/// - **Stopped** (T): A task will react to `SIGSTOP` or `SIGTSTP` signals and be brought back
/// to running or runnable by `SIGCONT` signal.
/// - **Zombie** (Z): When a task has completed its execution or is terminated, it will send the
/// `SIGCHLD` signal to the parent task and go into the zombie state.
pub enum TaskState {
    Runnable,
    Running,
    Stopped,
    Interruptible,
    Uninterruptible,
    Zombie,
}

pub type TASK = Arc<Mutex<Task>>;

/// In conventional opinion, process is the minimum unit of resource allocation, while task (or
/// thread) is the minimum unit of scheduling. Process is always created with a main task. On
/// the one hand, a process may have several tasks; on the other hand, these tasks shared the
/// same information belonging to the process, such as virtual memory handler, process
/// identification (called pid) and etc.
///
/// We thus only use a shared region to hold the duplicated information of the tasks.
pub struct Task {
    /// Process identification
    pub pid: usize,

    /// Task identification
    pub tid: usize,

    /// TID allocator
    pub tid_allocator: RecycleAllocator,

    /// Task context
    pub ctx: TaskContext,

    /// Task state, using five-state model.
    pub state: TaskState,

    /// Memory
    pub mm: MM,

    /// Task exit code, known as the number returned to a parent process by an executable.
    pub exit_code: i32,

    /// Hierarchy pointers in task management.
    /// INIT task has no parent task.
    pub parent: Option<Arc<Mutex<Task>>>,

    /// Pointers to child tasks.
    /// When a parent task exits before its children, they will become orphans.
    /// These tasks will be adopted by INIT task to avoid being dropped when the reference
    /// counter becomes 0.
    pub children: Vec<Arc<Mutex<Task>>>,
}

impl Task {
    /// Create a new task with pid and kernel stack allocated by global manager.
    pub fn new(pid: usize, kstack: usize, elf_data: &[u8]) -> KernelResult<Self> {
        let mut tid_allocator = RecycleAllocator::new();
        let tid = tid_allocator.alloc();
        Ok(Self {
            pid,
            tid,
            tid_allocator,
            ctx: TaskContext::new(user_trap_return as usize, kstack_alloc(kstack)?),
            state: TaskState::Runnable,
            mm: from_elf(elf_data)?,
            exit_code: 0,
            parent: None,
            children: Vec::new(),
        })
    }

    /// Allocate a user stack in the same address space.
    /// User stack grows from high address to low address.
    ///
    /// Returns user stack base.
    pub fn ustack_alloc(&mut self, tid: usize) -> KernelResult<usize> {
        let ustack_base = ustack_base(tid);
        let ustack_top = ustack_base - USER_STACK_SIZE;
        self.mm.alloc_write(
            None,
            ustack_top.into(),
            ustack_base.into(),
            PTEFlags::READABLE | PTEFlags::WRITABLE,
            Arc::new(Mutex::new(FixedPMA::new(USER_STACK_PAGES)?)),
        )?;
        Ok(ustack_base)
    }
}


/// Returns trapframe base of the task in the address space by task identification.
/// 
/// Trapframes are located right below the Trampoline in each address space.
pub fn trapframe_base(tid: usize) -> usize {
    TRAMPOLINE_VA - PAGE_SIZE - tid * PAGE_SIZE
}

/// Returns user stack base of the task in the address space by task identification.
pub fn ustack_base(tid: usize) -> usize {
    USER_STACK_BASE - tid * (USER_STACK_SIZE + PAGE_SIZE)
}
