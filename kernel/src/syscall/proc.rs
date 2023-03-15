use alloc::{string::String, vec::Vec};
use errno::Errno;
use syscall_interface::*;
use vfs::{OpenFlags, Path};

use crate::{
    arch::{__move_to_next, mm::VirtAddr},
    fs::open,
    mm::{do_brk, do_mmap, do_mprotect, do_munmap, MmapFlags, MmapProt},
    read_user,
    task::*,
};

use super::SyscallImpl;

impl SyscallProc for SyscallImpl {
    fn clone(flags: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> SyscallResult {
        let flags = CloneFlags::from_bits(flags as u32);
        if flags.is_none() {
            return Err(Errno::EINVAL);
        }

        do_clone(
            flags.unwrap(),
            stack,
            tls,
            VirtAddr::from(ptid),
            VirtAddr::from(ctid),
        )
    }

    fn exit(status: usize) -> ! {
        unsafe { do_exit(status as i32) };
        unreachable!()
    }

    fn wait4(pid: isize, wstatus: usize, options: usize, rusage: usize) -> SyscallResult {
        let options = WaitOptions::from_bits(options as u32);
        if options.is_none() {
            return Err(Errno::EINVAL);
        }
        let options = options.unwrap();
        if !options
            .difference(
                WaitOptions::WNONHANG
                    | WaitOptions::WUNTRACED
                    | WaitOptions::WCONTINUED
                    | WaitOptions::__WALL
                    | WaitOptions::__WNOTHREAD
                    | WaitOptions::__WCLONE,
            )
            .is_empty()
        {
            return Err(Errno::EINVAL);
        }

        do_wait(pid, options | WaitOptions::WEXITED, 0, wstatus, rusage)
    }

    fn prlimit64(pid: isize, resource: i32, new_limit: usize, old_limit: usize) -> SyscallResult {
        if pid == 0 {
            do_prlimit(resource, new_limit, old_limit)
        } else {
            Err(Errno::ESRCH)
        }
    }

    fn execve(pathname: usize, argv: usize, _envp: usize) -> SyscallResult {
        let curr = cpu().curr.as_ref().unwrap();

        // get relative path under current working directory
        let rela_path = curr.mm().get_str(VirtAddr::from(pathname))?;

        // get absolute path of the file to execute
        let fs_info = curr.fs_info.lock();
        let mut path = Path::from(fs_info.cwd.clone() + "/" + rela_path.as_str());
        drop(fs_info);

        // read file from disk
        let file = open(path.clone(), OpenFlags::O_RDONLY)?;
        if !file.is_reg() {
            return Err(Errno::EACCES);
        }
        let elf_data = unsafe { file.read_all() };

        // get argument list
        let mut args = Vec::new();
        let mut argv = argv;
        let mut argc: usize = 0;
        let mut curr_mm = curr.mm();
        loop {
            read_user!(curr_mm, VirtAddr::from(argv), argc, usize)?;
            if argc == 0 {
                break;
            }
            args.push(curr_mm.get_str(VirtAddr::from(argc))?);
            argv += core::mem::size_of::<usize>();
        }
        drop(curr_mm);

        path.pop().unwrap(); // unwrap a regular filename freely
        do_exec(String::from(path.as_str()), elf_data.as_slice(), args)?;

        unsafe { __move_to_next(curr_ctx()) };

        unreachable!()
    }

    fn getpid() -> SyscallResult {
        Ok(cpu().curr.as_ref().unwrap().pid)
    }

    fn gettid() -> SyscallResult {
        Ok(cpu().curr.as_ref().unwrap().tid.0)
    }

    fn set_tid_address(tidptr: usize) -> SyscallResult {
        let curr = cpu().curr.as_ref().unwrap();
        curr.inner().clear_child_tid = tidptr;
        Ok(curr.tid.0)
    }

    fn brk(brk: usize) -> SyscallResult {
        do_brk(&mut cpu().curr.as_ref().unwrap().mm(), brk.into())
    }

    fn munmap(addr: usize, len: usize) -> SyscallResult {
        do_munmap(&mut cpu().curr.as_ref().unwrap().mm(), addr.into(), len)?;
        Ok(0)
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

        do_mmap(
            cpu().curr.as_ref().unwrap(),
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

        do_mprotect(
            &mut cpu().curr.as_ref().unwrap().mm(),
            addr.into(),
            len,
            prot.unwrap(),
        )?;
        Ok(0)
    }
}
