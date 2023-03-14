use errno::Errno;
use syscall_interface::{SyscallIO, SyscallResult};

use crate::{arch::mm::VirtAddr, task::cpu};

use super::SyscallImpl;

impl SyscallIO for SyscallImpl {
    fn ioctl(fd: usize, _request: usize, argp: *const usize) -> SyscallResult {
        let curr = cpu().curr.as_ref().unwrap();

        if curr.files().get(fd).is_err() {
            return Err(Errno::EBADF);
        }

        if curr
            .mm()
            .get_vma(VirtAddr::from(argp as usize), |_, _, _| Ok(()))
            .is_err()
        {
            return Err(Errno::EFAULT);
        }

        Ok(0)
    }
}
