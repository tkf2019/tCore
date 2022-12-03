use tsyscall::*;

use crate::task::manager::current_task;

use super::SyscallImpl;

impl SyscallInfo for SyscallImpl {
    fn getpid() -> SyscallResult {
        Ok(current_task().unwrap().pid.0)
    }

    fn gettid() -> SyscallResult {
        Ok(current_task().unwrap().tid)
    }

    fn set_tid_address(tidptr: usize) -> SyscallResult {
        let current = current_task().unwrap();
        current.inner_lock().clear_child_tid = tidptr;
        Ok(current.tid)
    }
}
