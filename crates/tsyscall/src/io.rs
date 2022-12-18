use crate::SyscallResult;

pub trait SyscallIO {
    /// Manipulates the underlying device parameters of special files.
    ///
    /// # Error
    /// - `EBADF`: fd is not a valid file descriptor.
    /// - `EFAULT`: argp references an inaccessible memory area.
    /// - `EINVAL`: request or argp is not valid.
    fn ioctl(fd: usize, request: usize, argp: *const usize) -> SyscallResult;
}
