use terrno::Errno;
use tmm_rv::VirtAddr;
use tsyscall::*;

use crate::task::current_task;

use super::SyscallImpl;

impl SyscallFile for SyscallImpl {
    fn write(fd: usize, buf: *const u8, count: usize) -> SyscallResult {
        let current = current_task().unwrap();
        let mut current_mm = current.mm.lock();

        // Translate user buffer into kernel string.
        let string = current_mm
            .get_str(VirtAddr::from(buf as usize), count)
            .map_err(|_| Errno::EFAULT)?;
        let bytes = string.as_bytes();
        drop(current_mm);

        // Get the file with the given file descriptor.
        let file = current
            .fd_manager
            .lock()
            .get(fd)
            .map_err(|_| Errno::EBADF)?;
        drop(current);

        if let Some(count) = file.write(bytes) {
            Ok(count)
        } else {
            Err(Errno::EINVAL)
        }
    }

    fn close(fd: usize) -> SyscallResult {
        todo!()
    }

    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult {
        todo!()
    }
}
