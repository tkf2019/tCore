use terrno::Errno;
use tmm_rv::VirtAddr;
use tsyscall::*;
use tvfs::{OpenFlags, StatMode};

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

    fn read(fd: usize, buf: *mut u8, count: usize) -> SyscallResult {
        let current = current_task().unwrap();
        let mut current_mm = current.mm.lock();

        // Get the real buffer translated into physical address.
        let buf = unsafe { current_mm.get_buf_mut(VirtAddr::from(buf as usize), count) }
            .map_err(|_| Errno::EFAULT)?;
        drop(current_mm);

        // Get the file with the given file descriptor.
        let file = current
            .fd_manager
            .lock()
            .get(fd)
            .map_err(|_| Errno::EBADF)?;
        drop(current);

        let mut count = 0;
        for buf in buf {
            if let Some(c) = file.read(buf) {
                count += c;
            } else {
                return Err(Errno::EINVAL);
            }
        }
        Ok(count)
    }

    fn close(fd: usize) -> SyscallResult {
        todo!()
    }

    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult {
        let flags = OpenFlags::from_bits(flags as u32);
        let mode = StatMode::from_bits(mode as u32);
        if flags.is_none() {
            return Err(Errno::EINVAL);
        }

        let current = current_task().unwrap();
        current.do_open(dirfd, pathname, flags.unwrap(), mode)
    }
}
