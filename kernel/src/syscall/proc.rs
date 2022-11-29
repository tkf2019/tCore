use tsyscall::*;

use crate::task::do_exit;

use super::SyscallImpl;

impl SyscallProc for SyscallImpl {
    fn clone(flags: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> SyscallResult {
        todo!()
    }

    fn exit(status: usize) -> ! {
        do_exit(status as i32);
        unreachable!()
    }

    fn execve(pathname: usize, argv: usize, envp: usize) -> SyscallResult {
        todo!()
    }
}
