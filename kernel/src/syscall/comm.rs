use core::mem::size_of;

use alloc::sync::Arc;
use terrno::Errno;
use tmm_rv::VirtAddr;
use tsyscall::{SyscallComm, SyscallResult};

use crate::{fs::Pipe, task::current_task};

use super::SyscallImpl;

impl SyscallComm for SyscallImpl {
    fn pipe(pipefd: *const u32, _flags: usize) -> SyscallResult {
        let current = current_task().unwrap();

        let mut fd_manager = current.fd_manager.lock();
        let (pipe_read, pipe_write) = Pipe::new();

        if fd_manager.fd_count() + 2 > fd_manager.fd_limit() {
            return Err(Errno::EMFILE);
        }

        let fd_read = fd_manager.push(Arc::new(pipe_read)).unwrap();
        let fd_write = fd_manager.push(Arc::new(pipe_write)).unwrap();
        drop(fd_manager);

        let mut mm = current.mm.lock();

        let fd_size = size_of::<u32>();
        let fd_addr = VirtAddr::from(pipefd as usize);
        if fd_addr.value() & (fd_size - 1) != 0 {
            return Err(Errno::EFAULT);
        }

        let buf = mm
            .get_buf_mut(fd_addr, 2 * fd_size)
            .map_err(|_| Errno::EFAULT)?;
        drop(mm);

        let mut iter = buf.into_iter().step_by(fd_size);
        let fd_read_ptr = unsafe { &mut *(iter.next().unwrap() as *mut u32) };
        let fd_write_ptr = unsafe { &mut *(iter.next().unwrap() as *mut u32) };
        *fd_read_ptr = fd_read as u32;
        *fd_write_ptr = fd_write as u32;

        Ok(0)
    }
}
