use errno::Errno;
use syscall_interface::{SyscallIO, SyscallResult};

use crate::{arch::mm::VirtAddr, task::curr_task};

use super::SyscallImpl;

impl SyscallIO for SyscallImpl {
    fn ioctl(fd: usize, _request: usize, argp: *const usize) -> SyscallResult {
        let curr = curr_task().unwrap();

        if curr.fd_manager.lock().get(fd).is_err() {
            return Err(Errno::EBADF);
        }

        if curr
            .mm
            .lock()
            .get_vma(VirtAddr::from(argp as usize), |_, _, _| Ok(()))
            .is_err()
        {
            return Err(Errno::EFAULT);
        }

        Ok(0)
    }
}
