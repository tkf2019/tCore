use tsyscall::*;

use super::SyscallImpl;

impl SyscallProc for SyscallImpl {
    fn clone(flags: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> SyscallResult {
        todo!()
    }

    fn exit(status: usize) -> ! {
        todo!()
    }

    fn execve(pathname: usize, argv: usize, envp: usize) -> SyscallResult {
        todo!()
    }
}
