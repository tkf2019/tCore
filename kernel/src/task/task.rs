use alloc::{rc::Weak, sync::Arc, vec::Vec};
use log::warn;
use riscv::register::sstatus::{self, set_spp, SPP};
use spin::{mutex::Mutex, MutexGuard};
use talloc::{IDAllocator, RecycleAllocator};
use tmm_rv::{PTEFlags, PhysAddr, VirtAddr, PAGE_SIZE};

use crate::{
    config::{
        ADDR_ALIGN, MAIN_TASK, TRAMPOLINE_VA, USER_STACK_BASE, USER_STACK_PAGES, USER_STACK_SIZE,
    },
    error::{KernelError, KernelResult},
    mm::{from_elf, pma::FixedPMA, KERNEL_MM, MM},
    trap::{user_trap_handler, user_trap_return, TrapFrame},
};

use super::{
    context::TaskContext,
    manager::{kstack_dealloc, kstack_vm_alloc, PID},
};

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

/// Mutable data owned by the task.
pub struct TaskInner {
    /// Task exit code, known as the number returned to a parent process by an executable.
    pub exit_code: i32,

    /// Task context
    pub ctx: TaskContext,

    /// Task state, using five-state model.
    pub state: TaskState,

    /// Hierarchy pointers in task management.
    /// INIT task has no parent task.
    pub parent: Option<Weak<Task>>,

    /// Pointers to child tasks.
    /// When a parent task exits before its children, they will become orphans.
    /// These tasks will be adopted by INIT task to avoid being dropped when the reference
    /// counter becomes 0.
    pub children: Vec<Arc<Task>>,
}

unsafe impl Send for TaskInner {}

/// In conventional opinion, process is the minimum unit of resource allocation, while task (or
/// thread) is the minimum unit of scheduling. Process is always created with a main task. On
/// the one hand, a process may have several tasks; on the other hand, these tasks shared the
/// same information belonging to the process, such as virtual memory handler, process
/// identification (called pid) and etc.
///
/// We use four types of regions to maintain the task metadata:
/// - Shared and immutable: uses [`Arc<T>`]
/// - Shared and mutable: uses [`Arc<Mutex<T>>`]
/// - Local and immutable: data initialized once when task created
/// - Local and mutable: uses [`Mutex<TaskInner>`] to wrap the data together
pub struct Task {
    /* Local and immutable */
    /// Kernel stack identification.
    pub kstack: usize,

    /// Task identification.
    pub tid: usize,

    /// Trapframe physical address.
    pub trapframe_pa: PhysAddr,

    /* Local and mutable */
    /// Inner data wrapped by [`Mutex`].
    inner: Mutex<TaskInner>,

    /* Shared and immutable */
    /// Process identification.
    /// 
    /// Use `Arc` to track the ownership of pid. If all tasks holding
    /// this pid exit and parent process release the resources through `wait()`,
    /// the pid will be released.
    pub pid: Arc<PID>,

    /* Shared and mutable */
    /// Task identification allocator.
    pub tid_allocator: Arc<Mutex<RecycleAllocator>>,

    /// Address space metadata.
    pub mm: Arc<Mutex<MM>>,
}

impl Task {
    /// Create a new task with pid and kernel stack allocated by global manager.
    pub fn new(pid: usize, kstack: usize, elf_data: &[u8]) -> KernelResult<Self> {
        let mut mm = MM::new()?;
        let kstack_base = kstack_vm_alloc(kstack)?;
        let mut tid_allocator = RecycleAllocator::new(MAIN_TASK);
        let tid = tid_allocator.alloc();
        // Init address space
        from_elf(elf_data, &mut mm)?;
        // Init user stack
        let ustack_base = ustack_base(tid);
        let ustack_top = ustack_base - USER_STACK_SIZE;
        mm.alloc_write(
            None,
            ustack_top.into(),
            ustack_base.into(),
            PTEFlags::READABLE | PTEFlags::WRITABLE | PTEFlags::USER_ACCESSIBLE,
            Arc::new(Mutex::new(FixedPMA::new(USER_STACK_PAGES)?)),
        )?;

        // Init trapframe
        let trapframe_base: VirtAddr = trapframe_base(tid).into();
        mm.alloc_write(
            None,
            trapframe_base,
            trapframe_base + PAGE_SIZE,
            PTEFlags::READABLE | PTEFlags::WRITABLE,
            Arc::new(Mutex::new(FixedPMA::new(1)?)),
        )?;
        let trapframe_pa = mm.page_table.translate(trapframe_base).map_err(|e| {
            warn!("{}", e);
            KernelError::PageTableInvalid
        })?;
        let trapframe = TrapFrame::from(trapframe_pa);
        unsafe { set_spp(SPP::User) };
        *trapframe = TrapFrame::new(
            KERNEL_MM.lock().page_table.satp(),
            kstack_base - ADDR_ALIGN,
            user_trap_handler as usize,
            mm.entry.value(),
            sstatus::read(),
            ustack_base - ADDR_ALIGN,
        );

        let task = Self {
            kstack,
            tid,
            trapframe_pa,
            inner: Mutex::new(TaskInner {
                exit_code: 0,
                ctx: TaskContext::new(user_trap_return as usize, kstack_base),
                state: TaskState::Runnable,
                parent: None,
                children: Vec::new(),
            }),
            pid: Arc::new(PID(pid)),
            tid_allocator: Arc::new(Mutex::new(tid_allocator)),
            mm: Arc::new(Mutex::new(mm)),
        };
        Ok(task)
    }

    /// Trapframe of this task
    pub fn trapframe(&self) -> &'static mut TrapFrame {
        TrapFrame::from(self.trapframe_pa)
    }

    /// Acquire inner lock to modify the metadata in [`TaskInner`].
    pub fn inner_lock(&self) -> MutexGuard<TaskInner> {
        self.inner.lock()
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        kstack_dealloc(self.kstack);
        // We don't release the memory resource occupied by the kernel stack.
        // This memory area might be used agian when a new task calls for a
        // new kernel stack.

        self.tid_allocator.lock().dealloc(self.tid);
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
