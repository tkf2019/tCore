use errno::Errno;
use mm_rv::VirtAddr;
use syscall_interface::*;

use crate::{
    mm::{do_brk, do_mmap, do_mprotect, do_munmap, MmapFlags, MmapProt},
    task::{curr_task, do_clone, do_exit, CloneFlags},
};

use super::SyscallImpl;

impl SyscallProc for SyscallImpl {
    fn clone(flags: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> SyscallResult {
        let flags = CloneFlags::from_bits(flags as u32);
        if flags.is_none() {
            return Err(Errno::EINVAL);
        }

        let curr = curr_task().unwrap();
        do_clone(
            &curr,
            flags.unwrap(),
            stack,
            tls,
            VirtAddr::from(ptid),
            VirtAddr::from(ctid),
        )
        .map_err(|err| err.into())
    }

    fn exit(status: usize) -> ! {
        unsafe { do_exit(status as i32) };
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
        do_brk(&mut curr_mm, brk.into())
    }

    fn munmap(addr: usize, len: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();
        do_munmap(&mut curr_mm, addr.into(), len)
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
        do_mmap(
            &curr,
            addr.into(),
            len,
            prot.unwrap(),
            flags.unwrap(),
            fd,
            off,
        )
    }

    fn mprotect(addr: usize, len: usize, prot: usize) -> SyscallResult {
        let prot = MmapProt::from_bits(prot);
        if prot.is_none() {
            return Err(Errno::EINVAL);
        }

        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();
        do_mprotect(&mut curr_mm, addr.into(), len, prot.unwrap())
            .map(|_| 0)
            .map_err(|err| err.into())
    }
}
