use core::cell::SyncUnsafeCell;

use alloc::{collections::LinkedList, sync::Arc};
use id_alloc::*;
use kernel_sync::SpinLock;
use log::trace;
use signal_defs::{SigPending, SigSet};

use crate::{
    arch::{
        mm::VirtAddr,
        trap::{user_trap_return, TrapFrame},
        TaskContext,
    },
    config::MAIN_TASK,
    error::{KernelError, KernelResult},
};

use super::{
    init_trapframe, kstack_alloc, kstack_dealloc, kstack_vm_alloc, pid_alloc, schedule::Scheduler,
    Task, TaskInner, TaskLockedInner, TaskState, PID, TASK_MANAGER,
};

bitflags::bitflags! {
    pub struct CloneFlags: u32 {
        /// Set if vm shared between processes. In particular, memory writes performed by the calling process or
        /// by the child process are also visible in the other process.
        const CLONE_VM = 0x00000100;
        /// Set if fs info shared between processes which includes the root of the filesystem,
        /// the current working directory, and the umask.
        const CLONE_FS = 0x00000200;
        /// Set if file descriptor table shared between processes
        const CLONE_FILES = 0x00000400;
        /// Set if signal handlers and blocked signals shared
        const CLONE_SIGHAND = 0x00000800;
        /// Set if a pidfd should be placed in parent
        const CLONE_PIDFD = 0x00001000;
        /// Set if we want to let tracing continue on the child too
        const CLONE_PTRACE = 0x00002000;
        /// Set if the parent wants the child to wake it up on mm_release
        const CLONE_VFORK = 0x00004000;
        /// Set if we want to have the same parent as the cloner
        const CLONE_PARENT = 0x00008000;
        /// Set if in the same thread group
        const CLONE_THREAD = 0x00010000;
        /// If set, the cloned child is started in a new mount namespace, initialized with a copy of
        /// the namespace of the parent.
        const CLONE_NEWNS = 0x00020000;
        /// If set, create a new TLS for the child
        const CLONE_SETTLS = 0x00080000;
        /// Store the child thread ID at the location pointed to by `parent_tid`.
        const CLONE_PARENT_SETTID = 0x00100000;
        /// Clear the child thread ID at the location pointed to by `child_tid` in child's memory
        /// when child exits, and do a wakeup on the futex at that address.
        const CLONE_CHILD_CLEARTID = 0x00200000;
        /// Store the child thread ID at the location pointed to by `child_tid` in child's memory.
        const CLONE_CHILD_SETTID = 0x01000000;
        /// New cgroup namespace
        const CLONE_NEWCGROUP = 0x02000000;
        /// New utsname namespace
        const CLONE_NEWUTS = 0x04000000;
        /// New ipc namespace
        const CLONE_NEWIPC = 0x08000000;
        /// New user namespace
        const CLONE_NEWUSER	= 0x10000000;
        /// New pid namespace
        const CLONE_NEWPID = 0x20000000;
        /// New network namespace
        const CLONE_NEWNET = 0x40000000;
        /// Clone io context
        const CLONE_IO = 0x80000000;
    }
}

pub fn do_clone(
    task: &Arc<Task>,
    flags: CloneFlags,
    stack: usize,
    tls: usize,
    ptid: VirtAddr,
    ctid: VirtAddr,
) -> KernelResult<usize> {
    trace!("CLONE from {:?} {:?}", &task, flags);

    /*
     * Thread groups must share signals as well, and detached threads
     * can only be started up within the thread group.
     */
    if flags.contains(CloneFlags::CLONE_THREAD) && !flags.contains(CloneFlags::CLONE_SIGHAND) {
        return Err(KernelError::InvalidArgs);
    }

    /*
     * Shared signal handlers imply shared VM. By way of the above,
     * thread groups also imply shared VM. Blocking this case allows
     * for various simplifications in other code.
     */
    if flags.contains(CloneFlags::CLONE_SIGHAND) && !flags.contains(CloneFlags::CLONE_VM) {
        return Err(KernelError::InvalidArgs);
    }

    // Clone address space
    let mm = if flags.contains(CloneFlags::CLONE_VM) {
        task.mm.clone()
    } else {
        let orig = task.mm.lock();
        Arc::new(SpinLock::new(orig.clone()?))
    };

    // Init trapframe
    let trapframe_pa = {
        let mut mm = mm.lock();
        let trapframe_pa = init_trapframe(&mut mm)?;
        let trapframe = TrapFrame::from(trapframe_pa);
        trapframe.copy_from(TrapFrame::from(task.trapframe_pa), flags, stack, tls);
        trapframe_pa
    };

    // New kernel stack
    let kstack = kstack_alloc();
    let kstack_base = kstack_vm_alloc(kstack);
    if kstack_base.is_err() {
        kstack_dealloc(kstack);
        return Err(kstack_base.unwrap_err());
    }

    // New task id
    let tid = if flags.contains(CloneFlags::CLONE_THREAD) {
        task.tid_allocator.lock().alloc()
    } else {
        MAIN_TASK
    };

    let new_task = Arc::new(Task {
        name: task.name.clone(),
        kstack,
        tid,
        trapframe_pa,
        pid: if flags.contains(CloneFlags::CLONE_THREAD) {
            task.pid.clone()
        } else {
            Arc::new(PID(pid_alloc()))
        },
        tid_allocator: if flags.contains(CloneFlags::CLONE_THREAD) {
            task.tid_allocator.clone()
        } else {
            Arc::new(SpinLock::new(RecycleAllocator::new(MAIN_TASK + 1)))
        },
        mm,
        fd_manager: if flags.contains(CloneFlags::CLONE_FILES) {
            task.fd_manager.clone()
        } else {
            let orig = task.fd_manager.lock();
            Arc::new(SpinLock::new(orig.clone()))
        },
        fs_info: if flags.contains(CloneFlags::CLONE_FS) {
            task.fs_info.clone()
        } else {
            let orig = task.fs_info.lock();
            Arc::new(SpinLock::new(orig.clone()))
        },
        sig_actions: if flags.contains(CloneFlags::CLONE_SIGHAND | CloneFlags::CLONE_THREAD) {
            task.sig_actions.clone()
        } else {
            let orig = task.sig_actions.lock();
            Arc::new(SpinLock::new(orig.clone()))
        },
        locked_inner: SpinLock::new(TaskLockedInner {
            state: TaskState::RUNNABLE,
            sleeping_on: None,
            parent: if flags.contains(CloneFlags::CLONE_PARENT) {
                let locked = task.locked_inner();
                locked.parent.clone()
            } else {
                Some(Arc::downgrade(task))
            },
            children: LinkedList::new(),
        }),
        inner: SyncUnsafeCell::new(TaskInner {
            exit_code: 0,
            ctx: TaskContext::new(user_trap_return as usize, kstack_base.unwrap()),
            set_child_tid: if flags.contains(CloneFlags::CLONE_CHILD_SETTID) {
                ctid.value()
            } else {
                0
            },
            clear_child_tid: if flags.contains(CloneFlags::CLONE_CHILD_CLEARTID) {
                ctid.value()
            } else {
                0
            },
            sig_pending: SigPending::new(),
            sig_blocked: SigSet::new(),
        }),
    });

    let pid = new_task.pid.0;

    // Set tid in parent address space
    if flags.contains(CloneFlags::CLONE_PARENT_SETTID) {
        let mut mm = task.mm.lock();
        let ptid = mm.alloc_frame(ptid)?.start_address() + ptid.page_offset();
        unsafe { *(ptid.get_mut() as *mut i32) = tid as i32 };
    }

    // Set tid in child address space (COW)
    if flags.contains(CloneFlags::CLONE_CHILD_SETTID | CloneFlags::CLONE_CHILD_CLEARTID) {
        let mut mm = new_task.mm.lock();
        let ctid = if flags.contains(CloneFlags::CLONE_VM) {
            mm.alloc_frame(ctid)?.start_address() + ctid.page_offset()
        } else {
            mm.force_alloc_frame(ctid)?.start_address() + ctid.page_offset()
        };
        unsafe {
            *(ctid.get_mut() as *mut i32) = if flags.contains(CloneFlags::CLONE_CHILD_SETTID) {
                tid as i32
            } else {
                0
            }
        };
    }

    /* New task will not be dropped from now on. */

    TASK_MANAGER.lock().add(new_task.clone());

    // we don't need to lock the new task
    let locked = unsafe { &mut *new_task.locked_inner.as_mut_ptr() };
    if let Some(parent) = &locked.parent {
        if let Some(parent) = parent.upgrade() {
            let mut locked = parent.locked_inner();
            locked.children.push_back(new_task);
        }
    }

    Ok(pid)
}
