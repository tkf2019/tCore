use core::cell::SyncUnsafeCell;

use alloc::{collections::LinkedList, string::String, sync::Arc, vec::Vec};
use errno::Errno;
use kernel_sync::SpinLock;
use mm_rv::{Frame, PTEFlags, Page};
use signal_defs::*;
use syscall_interface::SyscallResult;

use crate::{
    arch::{
        mm::VirtAddr,
        trap::{user_trap_handler, user_trap_return, TrapFrame},
        TaskContext,
    },
    error::*,
    loader::from_elf,
    mm::{KERNEL_MM, MM},
    task::{TrapFrameTracker, TID},
};

#[cfg(feature = "uintr")]
use crate::arch::uintr::*;

use super::*;

bitflags::bitflags! {
    /// A bit mask that allows the caller to specify what is shared between the calling process and the child process.
    pub struct CloneFlags: u32 {
        /// Signal mask to be sent at exit.
        const CSIGNAL = 0x000000ff;
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

/// A helper for [`syscall_interface::SyscallProc::clone`]
pub fn do_clone(
    flags: CloneFlags,
    stack: usize,
    tls: usize,
    ptid: VirtAddr,
    ctid: VirtAddr,
) -> SyscallResult {
    let curr = cpu().curr.as_ref().unwrap();
    log::trace!("CLONE {:?} {:?}", &curr, flags);

    if flags.intersects(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_FS) {
        return Err(Errno::EINVAL);
    }

    /*
     * Thread groups must share signals as well, and detached threads
     * can only be started up within the thread group.
     */
    if flags.contains(CloneFlags::CLONE_THREAD) && !flags.contains(CloneFlags::CLONE_SIGHAND) {
        return Err(Errno::EINVAL);
    }

    /*
     * Shared signal handlers imply shared VM. By way of the above,
     * thread groups also imply shared VM. Blocking this case allows
     * for various simplifications in other code.
     */
    if flags.contains(CloneFlags::CLONE_SIGHAND) && !flags.contains(CloneFlags::CLONE_VM) {
        return Err(Errno::EINVAL);
    }

    // Clone address space
    let mm = if flags.contains(CloneFlags::CLONE_VM) {
        curr.inner().mm.clone()
    } else {
        Arc::new(SpinLock::new(curr.mm().clone()?))
    };

    // New kernel stack
    let kstack = kstack_alloc();
    let tid = TID(kstack);
    let kstack_base = kstack_vm_alloc(kstack)?;

    // Init trapframe
    let trapframe_pa = {
        let mut mm = mm.lock();
        let trapframe_pa = init_trapframe(&mut mm, kstack)?;
        let trapframe = TrapFrame::from(trapframe_pa);
        trapframe.copy_from(curr.trapframe(), flags, stack, tls, kstack_base);
        trapframe_pa
    };

    let new_task = Arc::new(Task {
        name: curr.name.clone() + " (CLONED)",
        tid,
        /*
         * When a clone call is made without specifying CLONE_THREAD,
         * then the resulting thread is placed in a new thread group
         * whose TGID is the same as the thread's TID. This thread
         * is the leader of the new thread group.
         */
        pid: if flags.contains(CloneFlags::CLONE_THREAD) {
            curr.pid
        } else {
            kstack
        },
        trapframe: Some(TrapFrameTracker(trapframe_pa)),
        exit_signal: if flags.contains(CloneFlags::CLONE_THREAD) {
            SIGNONE
        } else {
            let sig = (flags & CloneFlags::CSIGNAL).bits() as usize;
            if !sigvalid(sig) {
                return Err(Errno::EINVAL);
            }
            sig
        },
        fs_info: if flags.contains(CloneFlags::CLONE_FS) {
            curr.fs_info.clone()
        } else {
            let orig = curr.fs_info.lock();
            Arc::new(SpinLock::new(orig.clone()))
        },
        sig_actions: if flags.intersects(CloneFlags::CLONE_SIGHAND | CloneFlags::CLONE_THREAD) {
            curr.sig_actions.clone()
        } else {
            let orig = curr.sig_actions.lock();
            Arc::new(SpinLock::new(orig.clone()))
        },
        locked_inner: SpinLock::new(TaskLockedInner {
            state: TaskState::RUNNABLE,
            sleeping_on: None,
            parent: if flags.intersects(CloneFlags::CLONE_PARENT | CloneFlags::CLONE_THREAD) {
                let locked = curr.locked_inner();
                locked.parent.clone()
            } else {
                Some(Arc::downgrade(&curr))
            },
            children: LinkedList::new(),
        }),
        inner: SyncUnsafeCell::new(TaskInner {
            exit_code: 0,
            ctx: TaskContext::new(user_trap_return as usize, kstack_base),
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
            mm,
            files: if flags.contains(CloneFlags::CLONE_FILES) {
                curr.inner().files.clone()
            } else {
                Arc::new(SpinLock::new(curr.files().clone()))
            },
        }),
        #[cfg(feature = "uintr")]
        uintr_inner: SyncUnsafeCell::new(TaskUIntrInner::new()),
    });

    // Set tid in parent address space
    if flags.contains(CloneFlags::CLONE_PARENT_SETTID) {
        let ptid = curr.mm().alloc_frame(ptid)?.start_address() + ptid.page_offset();
        unsafe { *(ptid.get_mut() as *mut i32) = kstack as i32 };
    }

    // Set tid in child address space (COW)
    if flags.intersects(CloneFlags::CLONE_CHILD_SETTID | CloneFlags::CLONE_CHILD_CLEARTID) {
        let ctid = new_task.mm().alloc_frame(ctid)?.start_address() + ctid.page_offset();
        unsafe {
            *(ctid.get_mut() as *mut i32) = if flags.contains(CloneFlags::CLONE_CHILD_SETTID) {
                kstack as i32
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
            let mut parent = parent.locked_inner();
            parent.children.push_back(new_task);
        }
    }

    Ok(kstack)
}

/// A helper for [`syscall_interface::SyscallProc::execve`]
pub fn do_exec(dir: String, elf_data: &[u8], args: Vec<String>) -> KernelResult {
    let curr = cpu().curr.as_ref().unwrap();
    log::trace!("EXEC {:?} DIR [{}] {:?}", &curr, &dir, &args);

    // memory mappings are not preserved
    let mut mm = MM::new()?;
    let sp = from_elf(elf_data, args, &mut mm)?;

    // re-initialize trapframe
    let kstack_base = kstack_layout(curr.tid.0).1;
    let trapframe = curr.trapframe();
    *trapframe = TrapFrame::new(
        KERNEL_MM.lock().page_table.satp(),
        kstack_base,
        user_trap_handler as usize,
        mm.entry.value(),
        sp.into(),
    );
    mm.page_table
        .map(
            Page::from(VirtAddr::from(trapframe_base(curr.tid.0))),
            Frame::from(curr.trapframe.as_ref().unwrap().0),
            PTEFlags::READABLE | PTEFlags::WRITABLE | PTEFlags::VALID,
        )
        .map_err(|_| KernelError::PageTableInvalid)?;
    curr.inner().mm = Arc::new(SpinLock::new(mm));

    // the dispositions of any signals that are being caught are reset to the default
    *curr.sig_actions.lock() = [SigAction::default(); NSIG];

    /*
     * The file descriptor table is unshared, undoing the effect of the
     * CLONE_FILES flag of clone(2). By default, file descriptors remain
     * open across an execve(). File descriptors that are marked close-on-exec
     * are closed; see the description of FD_CLOEXEC in fcntl(2).
     */
    curr.inner().files = Arc::new(SpinLock::new(curr.files().clone()));
    unsafe { &mut *curr.inner().files.as_mut_ptr() }.cloexec();

    curr.inner().ctx = TaskContext::new(user_trap_return as usize, kstack_base);

    #[cfg(feature = "uintr")]
    {
        curr.uintr_inner().uist = Some(UIntrSender::new(1));
        curr.uintr_inner().uirs = Some(UIntrReceiverTracker::new());
        curr.uintr_inner().mask = 0;
    }

    Ok(())
}
