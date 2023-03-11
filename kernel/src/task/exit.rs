use alloc::sync::Arc;
use oscomp::finish_test;

use crate::arch::{TaskContext, __switch};

use super::{curr_task, idle_ctx, Task, TaskState, INIT_TASK};

/// Current task exits. Run next task.
///
/// # Safety
///
/// Unsafe context switch will be called in this function.
pub unsafe fn do_exit(exit_code: i32) {
    let curr = curr_task().take().unwrap();
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
        let mut child_inner = child.locked_inner();
        let mut init_task_inner = INIT_TASK.locked_inner();
        child_inner.parent = Some(Arc::downgrade(&INIT_TASK));
        init_task_inner.children.push_back(child.clone());
    }
    locked_inner.children.clear();
    locked_inner.state = TaskState::ZOMBIE;

    if let Some(parent) = &locked_inner.parent {
        let parent = parent.upgrade().unwrap();
        let mut locked_inner = parent.locked_inner();
        locked_inner
            .children
            .drain_filter(|child| child.tid == task.tid);
    }

    #[cfg(feature = "test")]
    if task.tid.0 == task.pid {
        finish_test(task.inner().exit_code, &task.name);
    }
}
