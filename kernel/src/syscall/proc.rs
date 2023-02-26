use errno::Errno;
use syscall_interface::*;

use crate::{
    mm::{MmapFlags, MmapProt},
    task::{curr_task, do_exit},
};

use super::SyscallImpl;

impl SyscallProc for SyscallImpl {
    fn clone(flags: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> SyscallResult {
        todo!()
    }

    fn exit(status: usize) -> ! {
        do_exit(status as i32);
        unreachable!()
    }

    fn execve(pathname: usize, argv: usize, envp: usize) -> SyscallResult {
        todo!()
    }

    fn getpid() -> SyscallResult {
        Ok(curr_task().unwrap().pid.0)
    }

    fn gettid() -> SyscallResult {
        Ok(curr_task().unwrap().tid)
    }

    fn set_tid_address(tidptr: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        curr.inner().clear_child_tid = tidptr;
        Ok(curr.tid)
    }

    fn brk(brk: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();
        curr_mm.do_brk(brk.into())
    }

    fn munmap(addr: usize, len: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();
        curr_mm
            .do_munmap(addr.into(), len)
            .map(|_| 0)
            .map_err(|err| err.into())
    }

    fn mmap(
        addr: usize,
        len: usize,
        prot: usize,
        flags: usize,
        fd: usize,
        off: usize,
    ) -> SyscallResult {
        let prot = MmapProt::from_bits(prot);
        let flags = MmapFlags::from_bits(flags);
        if prot.is_none() || flags.is_none() {
            return Err(Errno::EINVAL);
        }

        let curr = curr_task().unwrap();
        curr.do_mmap(addr.into(), len, prot.unwrap(), flags.unwrap(), fd, off)
    }

    fn mprotect(addr: usize, len: usize, prot: usize) -> SyscallResult {
        let prot = MmapProt::from_bits(prot);
        if prot.is_none() {
            return Err(Errno::EINVAL);
        }

        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();
        curr_mm
            .do_mprotect(addr.into(), len, prot.unwrap())
            .map(|_| 0)
            .map_err(|err| err.into())
    }
}
