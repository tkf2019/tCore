use alloc::sync::Arc;
use core::mem::size_of;
use errno::Errno;
use signal_defs::{SigAction, SigActionFlags, SigSet, SignalNo, NSIG};
use syscall_interface::{SyscallComm, SyscallResult};

use crate::{arch::mm::VirtAddr, fs::Pipe, task::curr_task, user_buf_next, user_buf_next_mut};

use super::SyscallImpl;

impl SyscallComm for SyscallImpl {
    fn pipe(pipefd: *const u32, _flags: usize) -> SyscallResult {
        let curr = curr_task().unwrap();

        let mut fd_manager = curr.fd_manager.lock();
        let (pipe_read, pipe_write) = Pipe::new();

        if fd_manager.fd_count() + 2 > fd_manager.fd_limit() {
            return Err(Errno::EMFILE);
        }

        let fd_read = fd_manager.push(Arc::new(pipe_read)).unwrap();
        let fd_write = fd_manager.push(Arc::new(pipe_write)).unwrap();
        drop(fd_manager);

        let mut curr_mm = curr.mm.lock();

        let fd_size = size_of::<u32>();
        let fd_addr = VirtAddr::from(pipefd as usize);
        if fd_addr.value() & (fd_size - 1) != 0 {
            return Err(Errno::EFAULT);
        }

        let buf = curr_mm
            .get_buf_mut(fd_addr, 2 * fd_size)
            .map_err(|_| Errno::EFAULT)?;
        drop(curr_mm);

        let mut iter = buf.into_iter().step_by(fd_size);
        *user_buf_next_mut!(iter, u32) = fd_read as u32;
        *user_buf_next_mut!(iter, u32) = fd_write as u32;

        Ok(0)
    }

    fn sigaction(signum: usize, act: usize, oldact: usize) -> SyscallResult {
        if act != 0
            && (signum == SignalNo::SIGKILL.into()
                || signum == SignalNo::SIGSTOP.into()
                || signum >= NSIG)
        {
            return Err(Errno::EINVAL);
        }

        let sig_action_size = size_of::<SigAction>();
        if act & (sig_action_size - 1) != 0 || oldact & (sig_action_size - 1) != 0 {
            return Err(Errno::EINVAL);
        }

        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();
        let mut sig_actions = curr.sig_actions.lock();

        if oldact != 0 {
            let oldact = curr_mm
                .get_buf_mut(oldact.into(), sig_action_size)
                .map_err(|_| Errno::EFAULT)?;
            let mut iter = oldact.into_iter().step_by(size_of::<usize>());
            let old_sig_action = sig_actions.get_ref(signum);
            *user_buf_next_mut!(iter, usize) = old_sig_action.handler;
            *user_buf_next_mut!(iter, SigActionFlags) = old_sig_action.flags;
            *user_buf_next_mut!(iter, usize) = old_sig_action.restorer;
            *user_buf_next_mut!(iter, SigSet) = old_sig_action.mask;
        }

        if act != 0 {
            let act = curr_mm
                .get_buf_mut(act.into(), sig_action_size)
                .map_err(|_| Errno::EFAULT)?;
            let mut iter = act.into_iter().step_by(size_of::<usize>());
            let sig_action = sig_actions.get_mut(signum);
            sig_action.handler = *user_buf_next!(iter, usize);
            sig_action.flags = *user_buf_next!(iter, SigActionFlags);
            sig_action.restorer = *user_buf_next!(iter, usize);
            sig_action.mask = *user_buf_next!(iter, SigSet);
        }

        Ok(0)
    }

    fn sigpending(set: usize) -> SyscallResult {
        Ok(0)
    }

    fn sigprocmask(how: usize, set: usize, oldset: usize, sigsetsize: usize) -> SyscallResult {
        Ok(0)
    }
}
