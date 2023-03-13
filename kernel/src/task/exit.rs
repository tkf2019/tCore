use alloc::sync::Arc;
use errno::Errno;
use mm_rv::VirtAddr;
use oscomp::finish_test;
use signal_defs::*;
use syscall_interface::SyscallResult;

use crate::{
    arch::{TaskContext, __switch},
    error::KernelResult,
    write_user,
};

use super::{cpu, curr_ctx, curr_task, do_yield, idle_ctx, Task, TaskState, INIT_TASK};

/// Current task exits. Run next task.
///
/// # Safety
///
/// Unsafe context switch will be called in this function.
pub unsafe fn do_exit(exit_code: i32) {
    let curr = cpu().curr.take().unwrap();
    log::info!("{:?} exited with code {}", curr, exit_code);
    let curr_ctx = {
        let mut locked_inner = curr.locked_inner();
        curr.inner().exit_code = exit_code;
        locked_inner.state = TaskState::ZOMBIE;
        &curr.inner().ctx as *const TaskContext
    };

    handle_zombie(curr);

    __switch(curr_ctx, idle_ctx());
}

// Handle zombie tasks.
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
///
///
pub fn handle_zombie(task: Arc<Task>) {
    let mut locked_inner = task.locked_inner();
    for child in locked_inner.children.iter() {
        let mut child_inner = child.locked_inner();
        let mut init_task_inner = INIT_TASK.locked_inner();
        child_inner.parent = Some(Arc::downgrade(&INIT_TASK));
        init_task_inner.children.push_back(child.clone());
    }
    locked_inner.children.clear();
    locked_inner.state = TaskState::ZOMBIE;

    #[cfg(feature = "test")]
    if task.tid.0 == task.pid {
        finish_test(task.inner().exit_code, &task.name);
    }
}

bitflags::bitflags! {
    pub struct WaitOptions: u32 {
        /// Return immediately if no child has exited.
        const WNONHANG = 0x00000001;
        /// Also return if a child has stopped (but not traced via ptrace(2)).
        /// Status for traced children which have stopped is provided even if
        /// this option is not specified.
        const WUNTRACED = 0x00000002;
        /// Wait for children that have been stopped by a delivery of a signal.
        const WSTOPPED = 0x00000002;
        /// Wait for children that have terminated.
        const WEXITED = 0x00000004;
        /// Also return if a stopped child has been resumed by delivery of SIGCONT.
        const WCONTINUED = 0x00000008;
        /// Leave the child in a waitable state; a later wait call can be used to
        /// again retrieve the child status information.
        const WNOWAIT = 0x01000000;

        /* Linux specified */

        /// Do not wait for children of other threads in the same thread group.
        /// This was the default before Linux 2.4.
        const __WNOTHREAD = 0x20000000;
        /// Wait for all children, regardless of type ("clone" or "non-clone").
        const __WALL = 0x40000000;
        ///  Wait for "clone" children only.  If omitted, then wait for "non-clone"
        /// children only. (A "clone" child is one which delivers no signal, or a
        /// signal other than SIGCHLD to its parent upon termination.)  This option
        /// is ignored if __WALL is also specified.
        const __WCLONE = 0x80000000;
    }
}

/// Checks if a child satisfies the pid and options given by the calling process.
fn valid_child(pid: isize, options: WaitOptions, task: &Task) -> bool {
    if pid > 0 {
        if task.pid != pid as usize {
            return false;
        }
    }

    /*
     * Here we assume that all processes in the same process group.
     * Thus the calling process will wait for any process.
     */

    /*
     * Wait for all children (clone and not) if __WALL is set;
     * otherwise, wait for clone children *only* if __WCLONE is
     * set; otherwise, wait for non-clone children *only*.  (Note:
     * A "clone" child here is one that reports to its parent
     * using a signal other than SIGCHLD.)
     */
    if (task.exit_signal != SIGCHLD) ^ options.contains(WaitOptions::__WCLONE)
        && !options.contains(WaitOptions::__WALL)
    {
        return false;
    }

    true
}

/// A helper for [`syscall_interface::SyscallProc::wait4`].
pub fn do_wait(
    pid: isize,
    options: WaitOptions,
    infop: usize,
    stat_addr: usize,
    rusage: usize,
) -> SyscallResult {
    log::trace!("WAIT4 {} {:?}", pid, options);

    loop {
        let mut flag = false;
        let mut need_sched = false;
        let mut child: usize = 0;
        let curr = curr_task().unwrap();
        let mut locked = curr.locked_inner();
        for (index, task) in locked.children.iter().enumerate() {
            if !valid_child(pid, options, &task) {
                continue;
            }
            // a valid child exists but current task needs to suspend
            need_sched = true;

            let state = task.get_state();
            if state == TaskState::STOPPED {
                todo!()
            } else {
                if state == TaskState::DEAD {
                    continue;
                }
                if state == TaskState::ZOMBIE {
                    if !options.contains(WaitOptions::WEXITED) {
                        continue;
                    }
                    // a child with changed state exists
                    flag = true;
                    child = index;
                    break;
                }
                if !options.contains(WaitOptions::WCONTINUED) {
                    continue;
                }
            }
        }
        if !flag {
            if options.contains(WaitOptions::WNONHANG) || !need_sched {
                return Err(Errno::ECHILD);
            }

            // schedule current task
            drop(locked);
            drop(curr);
            unsafe { do_yield() };
        } else {
            // reclaim resources
            let child = locked.children.remove(child);

            // store status information
            if stat_addr != 0 {
                let mut curr_mm = curr.mm.lock();
                write_user!(
                    curr_mm,
                    VirtAddr::from(stat_addr),
                    (child.inner().exit_code << 8) as i32,
                    i32
                )?;
            }
        }
    }

    Ok(0)
}
