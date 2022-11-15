use tsyscall::*;

use crate::task::manager::current_task;

use super::SyscallImpl;

impl SyscallInfo for SyscallImpl {
    fn getpid() -> SyscallResult {
        Ok(current_task().unwrap().pid.0)
    }
}
