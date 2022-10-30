use alloc::{sync::Arc, vec::Vec};
use spin::mutex::Mutex;
use talloc::RecycleAllocator;

use crate::mm::MM;

use super::context::TaskContext;

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
    pub context: TaskContext,

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

impl Task {}
