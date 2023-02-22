use errno::Errno;
use syscall_interface::*;

use crate::{
    arch::mm::VirtAddr,
    mm::{MmapFlags, MmapProt},
    task::{current_task, do_exit},
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
        Ok(current_task().unwrap().pid.0)
    }

    fn gettid() -> SyscallResult {
        Ok(current_task().unwrap().tid)
    }

    fn set_tid_address(tidptr: usize) -> SyscallResult {
        let current = current_task().unwrap();
        current.inner_lock().clear_child_tid = tidptr;
        Ok(current.tid)
    }

    fn brk(brk: usize) -> SyscallResult {
        let current = current_task().unwrap();
        let mut current_mm = current.mm.lock();
        current_mm.do_brk(VirtAddr::from(brk))
    }

    fn munmap(addr: usize, len: usize) -> SyscallResult {
        let current = current_task().unwrap();
        let mut current_mm = current.mm.lock();
        current_mm
            .do_munmap(VirtAddr::from(addr), len)
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

        let current = current_task().unwrap();
        current.do_mmap(
            VirtAddr::from(addr),
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

        
        Ok(0)
    }
}
