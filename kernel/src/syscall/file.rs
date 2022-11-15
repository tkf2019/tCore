use tsyscall::*;

use super::SyscallImpl;

impl SyscallFile for SyscallImpl {
    fn write(fd: usize, buf: *mut u8, count: usize) -> SyscallResult {
        todo!()
    }

    fn close(fd: usize) -> SyscallResult {
        todo!()
    }

    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult {
        todo!()
    }
}
