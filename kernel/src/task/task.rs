use alloc::{sync::Arc, vec::Vec};
use log::warn;
use riscv::register::sstatus::{self, set_spp, SPP};
use spin::mutex::Mutex;
use talloc::{IDAllocator, RecycleAllocator};
use tmm_rv::{PTEFlags, PhysAddr, VirtAddr, PAGE_SIZE};

use crate::{
    config::{ADDR_ALIGN, TRAMPOLINE_VA, USER_STACK_BASE, USER_STACK_PAGES, USER_STACK_SIZE},
    error::{KernelError, KernelResult},
    mm::{from_elf, pma::FixedPMA, KERNEL_MM, MM},
    trap::{user_trap_handler, user_trap_return, TrapFrame},
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

    /// TID allocator: TODO
    pub tid_allocator: RecycleAllocator,

    /// Task context
    pub ctx: TaskContext,

    /// Trapframe physical address
    pub trapframe_pa: PhysAddr,

    /// Task state, using five-state model.
    pub state: TaskState,

    /// Memory
    pub mm: MM,

    /// Task exit code, known as the number returned to a parent process by an executable.
    pub exit_code: i32,

    /// Hierarchy pointers in task management.
    /// INIT task has no parent task.
    pub parent: Option<TASK>,

    /// Pointers to child tasks.
    /// When a parent task exits before its children, they will become orphans.
    /// These tasks will be adopted by INIT task to avoid being dropped when the reference
    /// counter becomes 0.
    pub children: Vec<TASK>,
}

impl Task {
    /// Create a new task with pid and kernel stack allocated by global manager.
    pub fn new(pid: usize, kstack: usize, elf_data: &[u8]) -> KernelResult<Self> {
        let mut tid_allocator = RecycleAllocator::new();
        let tid = tid_allocator.alloc();
        let kstack_base = kstack_alloc(kstack)?;
        let mut task = Self {
            pid,
            tid,
            tid_allocator,
            ctx: TaskContext::new(user_trap_return as usize, kstack_base),
            trapframe_pa: PhysAddr::zero(),
            state: TaskState::Runnable,
            mm: from_elf(elf_data)?,
            exit_code: 0,
            parent: None,
            children: Vec::new(),
        };
        // Init user stack
        let ustack_base = ustack_base(tid);
        let ustack_top = ustack_base - USER_STACK_SIZE;
        task.mm.alloc_write(
            None,
            ustack_top.into(),
            ustack_base.into(),
            PTEFlags::READABLE | PTEFlags::WRITABLE | PTEFlags::USER_ACCESSIBLE,
            Arc::new(Mutex::new(FixedPMA::new(USER_STACK_PAGES)?)),
        )?;
        // Init trapframe
        let trapframe_base: VirtAddr = trapframe_base(tid).into();
        task.mm.alloc_write(
            None,
            trapframe_base,
            trapframe_base + PAGE_SIZE,
            PTEFlags::READABLE | PTEFlags::WRITABLE,
            Arc::new(Mutex::new(FixedPMA::new(1)?)),
        )?;
        task.trapframe_pa = task.mm.page_table.translate(trapframe_base).map_err(|e| {
            warn!("{}", e);
            KernelError::PageTableInvalid
        })?;
        let trapframe = task.trapframe();
        // Init sstatus
        unsafe { set_spp(SPP::User) };
        *trapframe = TrapFrame::new(
            KERNEL_MM.lock().page_table.satp(),
            kstack_base - ADDR_ALIGN,
            user_trap_handler as usize,
            task.mm.entry.value(),
            sstatus::read(),
            ustack_base - ADDR_ALIGN,
        );
        Ok(task)
    }

    pub fn trapframe(&self) -> &mut TrapFrame {
        unsafe {
            (self.trapframe_pa.value() as *mut TrapFrame)
                .as_mut()
                .unwrap()
        }
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
