use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{fmt, mem::size_of};
use log::trace;
use riscv::register::sstatus::{self, set_spp, SPP};
use spin::{mutex::Mutex, MutexGuard};
use talloc::{IDAllocator, RecycleAllocator};
use terrno::Errno;
use tmm_rv::{PTEFlags, PhysAddr, VirtAddr, PAGE_SIZE};
use tsyscall::{IoVec, SyscallResult, AT_FDCWD, AT_REMOVEDIR};
use tvfs::{File, OpenFlags, Path, StatMode};

use crate::{
    config::*,
    error::{KernelError, KernelResult},
    fs::{open, unlink, FDManager},
    loader::from_elf,
    mm::{pma::FixedPMA, BackendFile, MmapFlags, MmapProt, KERNEL_MM, MM},
    task::{kstack_alloc, pid_alloc},
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
#[derive(Debug, Clone, Copy)]
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

    /// If a thread is started using `clone(2)` with the `CLONE_CHILD_SETTID` flag,
    /// set_child_tid is set to the value passed in the ctid argument of that system call.
    ///
    /// When set_child_tid is set, the very first thing the new thread does is to write
    /// its thread ID at this address.
    pub set_child_tid: usize,

    /// If a thread is started using `clone(2)` with the `CLONE_CHILD_CLEARTID` flag,
    /// clear_child_tid is set to the value passed in the ctid argument of that system call.
    pub clear_child_tid: usize,

    /// Current working directory.
    pub curr_dir: String,
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

    /// File descriptor table.
    pub fd_manager: Arc<Mutex<FDManager>>,

    /// Name of this task.
    pub name: String,
}

impl Task {
    /// Create a new task from ELF data.
    pub fn new(dir: String, elf_data: &[u8], args: Vec<String>) -> KernelResult<Self> {
        // Get task name
        let name = args[0].clone();

        // Init address space
        let mut mm = MM::new()?;
        let sp = from_elf(elf_data, args, &mut mm)?;
        trace!("\nTask [{}]\n{:#?}", &name, mm);

        // New process identification
        let pid = pid_alloc();

        // New kernel stack for user task
        let kstack = kstack_alloc();
        let kstack_base = kstack_vm_alloc(kstack)?;

        // Init trapframe
        let trapframe_base: VirtAddr = trapframe_base(MAIN_TASK).into();
        mm.alloc_write_vma(
            None,
            trapframe_base,
            trapframe_base + PAGE_SIZE,
            PTEFlags::READABLE | PTEFlags::WRITABLE,
            Arc::new(Mutex::new(FixedPMA::new(1)?)),
        )?;
        let trapframe_pa = mm.translate(trapframe_base)?;
        let trapframe = TrapFrame::from(trapframe_pa);
        unsafe { set_spp(SPP::User) };
        *trapframe = TrapFrame::new(
            KERNEL_MM.lock().page_table.satp(),
            kstack_base,
            user_trap_handler as usize,
            mm.entry.value(),
            sstatus::read(),
            sp.into(),
            // CPU id will be saved when the user task is restored.
            usize::MAX,
        );

        // Init file descriptor table
        let fd_manager = FDManager::new();

        let task = Self {
            kstack,
            tid: MAIN_TASK,
            trapframe_pa,
            inner: Mutex::new(TaskInner {
                exit_code: 0,
                ctx: TaskContext::new(user_trap_return as usize, kstack_base),
                state: TaskState::Runnable,
                parent: None,
                children: Vec::new(),
                set_child_tid: 0,
                clear_child_tid: 0,
                curr_dir: dir,
            }),
            pid: Arc::new(PID(pid)),
            tid_allocator: Arc::new(Mutex::new(RecycleAllocator::new(MAIN_TASK + 1))),
            mm: Arc::new(Mutex::new(mm)),
            fd_manager: Arc::new(Mutex::new(fd_manager)),
            name,
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

    /// Try to acquire inner lock to modify the metadata in [`TaskInner`]
    pub fn inner_try_lock(&self) -> Option<MutexGuard<TaskInner>> {
        self.inner.try_lock()
    }

    /// Get task status
    pub fn get_state(&self) -> TaskState {
        let inner = self.inner.lock();
        inner.state
    }

    /// Get the reference of a file object by file descriptor `fd`.
    pub fn get_file(&self, fd: usize) -> KernelResult<Arc<dyn File>> {
        let fd_manager = self.fd_manager.lock();
        fd_manager.get(fd)
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

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task [{} {}:{}]", self.name, self.pid.0, self.tid)
    }
}

/// Returns trapframe base of the task in the address space by task identification.
///
/// Trapframes are located right below the Trampoline in each address space.
pub fn trapframe_base(tid: usize) -> usize {
    TRAMPOLINE_VA - PAGE_SIZE - tid * PAGE_SIZE
}

/// Returns task stack layout [top, base) by task identification.
///
/// Stack grows from high address to low address.
pub fn ustack_layout(tid: usize) -> (usize, usize) {
    let ustack_base = USER_STACK_BASE - tid * (USER_STACK_SIZE + PAGE_SIZE);
    let ustack_top = ustack_base - USER_STACK_SIZE;
    (ustack_top, ustack_base - ADDR_ALIGN)
}

/* Syscall helpers */

impl Task {
    /// A helper for [`tsyscall::SyscallProc::mmap`].
    ///
    /// TODO: MAP_SHARED and MAP_PRIVATE
    pub fn do_mmap(
        &self,
        hint: VirtAddr,
        len: usize,
        prot: MmapProt,
        flags: MmapFlags,
        fd: usize,
        off: usize,
    ) -> SyscallResult {
        trace!(
            "MMAP {:?}, 0x{:X?} {:#?} {:#?} 0x{:X} 0x{:X}",
            hint,
            len,
            prot,
            flags,
            fd,
            off
        );

        if len == 0
            || !hint.is_aligned()
            || !(hint + len).is_aligned()
            || hint + len > VirtAddr::from(LOW_MAX_VA)
            || hint == VirtAddr::zero() && flags.contains(MmapFlags::MAP_FIXED)
        {
            return Err(Errno::EINVAL);
        }

        let mut mm = self.mm.lock();
        if mm.map_count() >= MAX_MAP_COUNT {
            return Err(Errno::ENOMEM);
        }

        // Find an available area by kernel.
        let anywhere = hint == VirtAddr::zero() && !flags.contains(MmapFlags::MAP_FIXED);

        // Handle different cases indicated by `MmapFlags`.
        if flags.contains(MmapFlags::MAP_ANONYMOUS) {
            if fd as isize == -1 && off == 0 {
                if let Ok(start) = mm.alloc_vma(hint, hint + len, prot.into(), anywhere, None) {
                    trace!("{:#?}", mm);
                    return Ok(start.value());
                } else {
                    return Err(Errno::ENOMEM);
                }
            }
            return Err(Errno::EINVAL);
        }

        // Map to backend file.
        if let Ok(file) = self.fd_manager.lock().get(fd) {
            if !file.is_reg() || !file.read_ready() {
                return Err(Errno::EACCES);
            }
            if let Some(_) = file.seek(off, tvfs::SeekWhence::Set) {
                let backend = BackendFile::new(file, off);
                if let Ok(start) =
                    mm.alloc_vma(hint, hint + len, prot.into(), anywhere, Some(backend))
                {
                    return Ok(start.value());
                } else {
                    return Err(Errno::ENOMEM);
                }
            } else {
                return Err(Errno::EACCES);
            }
        } else {
            return Err(Errno::EBADF);
        }

        // Invalid arguments or unimplemented cases
        // flags contained none of MAP_PRIVATE, MAP_SHARED, or MAP_SHARED_VALIDATE.
        // Err(Errno::EINVAL)
    }

    /// Gets the directory name from a file descriptor.
    pub fn get_dir(&self, dirfd: usize) -> KernelResult<Path> {
        if dirfd == AT_FDCWD {
            Ok(Path::new(self.inner_lock().curr_dir.as_str()))
        } else {
            let dir = self.fd_manager.lock().get(dirfd)?;
            if dir.is_dir() {
                Ok(dir.get_path().unwrap())
            } else {
                Err(KernelError::Errno(Errno::ENOTDIR))
            }
        }
    }

    /// Resolves absolute path with directory file descriptor and pathname.
    ///
    /// If the pathname is relative, then it is interpreted relative to the directory
    /// referred to by the file descriptor dirfd .
    ///
    /// If pathname is relative and dirfd is the special value [`AT_FDCWD`], then pathname
    /// is interpreted relative to the current working directory of the calling process.
    ///
    /// If pathname is absolute, then dirfd is ignored.
    pub fn resolve_path(&self, dirfd: usize, pathname: String) -> KernelResult<Path> {
        if pathname.starts_with("/") {
            Ok(Path::new(pathname.as_str()))
        } else {
            let mut path = self.get_dir(dirfd)?;
            path.extend(pathname.as_str());
            Ok(path)
        }
    }

    /// A helper for [`tsyscall::SyscallFile::openat`].
    pub fn do_open(
        &self,
        dirfd: usize,
        pathname: *const u8,
        flags: OpenFlags,
        mode: Option<StatMode>,
    ) -> KernelResult<usize> {
        if flags.contains(OpenFlags::O_CREAT) && mode.is_none()
            || flags.contains(OpenFlags::O_WRONLY | OpenFlags::O_RDWR)
        {
            return Err(KernelError::Errno(Errno::EINVAL));
        }

        let mut mm = self.mm.lock();
        let path = self.resolve_path(dirfd, mm.get_str(VirtAddr::from(pathname as usize))?)?;

        trace!("OPEN {:?} {:?}", path, flags);

        self.fd_manager
            .lock()
            .push(open(path, flags).map_err(|errno| KernelError::Errno(errno))?)
    }

    /// A helper for [`tsyscall::SyscallFile::readv`] and [`tsyscall::SyscallFile::writev`].
    pub fn for_each_iov(
        &self,
        iov: VirtAddr,
        iovcnt: usize,
        mut op: impl FnMut(usize, usize) -> bool,
    ) -> KernelResult {
        let size = size_of::<IoVec>();
        let mut mm = self.mm.lock();
        let buf = mm.get_buf_mut(iov, iovcnt * size)?;
        for bytes in buf.into_iter().step_by(size) {
            let iov = unsafe { &*(bytes as *const IoVec) };
            if !op(iov.iov_base, iov.iov_len) {
                break;
            }
        }
        Ok(())
    }

    pub fn do_unlinkat(&self, dirfd: usize, pathname: *const u8, flags: usize) -> KernelResult {
        if flags == AT_REMOVEDIR {
            unimplemented!()
        } else if flags == 0 {
            let mut mm = self.mm.lock();
            let path = self.resolve_path(dirfd, mm.get_str(VirtAddr::from(pathname as usize))?)?;

            trace!("UNLINKAT {:?}", path);

            unlink(path).map_err(|err| KernelError::Errno(err))
        } else {
            Err(KernelError::InvalidArgs)
        }
    }
}
