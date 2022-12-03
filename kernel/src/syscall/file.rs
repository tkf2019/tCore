use terrno::Errno;
use tmm_rv::VirtAddr;
use tsyscall::*;

use crate::{println, task::current_task};

use super::SyscallImpl;

impl SyscallFile for SyscallImpl {
    fn write(fd: usize, buf: *const u8, count: usize) -> SyscallResult {
        let current = current_task().unwrap();
        let mut current_mm = current.mm.lock();

        // Translate user buffer into kernel string.
        // EFAULT: buf is outside your accessible address space.
        let string = current_mm
            .page_table
            .get_str(VirtAddr::from(buf as usize), count)
            .map_err(|_| Errno::EFAULT)?;
        let bytes = string.as_bytes();
        drop(current_mm);

        // Get the file with the given file descriptor.
        // EBADF: fd is not a valid file descriptor or is not open for writing.
        let file = current
            .fd_manager
            .lock()
            .get(fd)
            .map_err(|_| Errno::EBADF)?;
        drop(current);

        // EINVAL: fd is attached to an object which is unsuitable for writing.
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
