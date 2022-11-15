use tmm_rv::VirtAddr;
use tsyscall::*;

use crate::{print, task::current_task};

use super::SyscallImpl;

impl SyscallFile for SyscallImpl {
    fn write(fd: usize, buf: *mut u8, count: usize) -> SyscallResult {
        let current = current_task().unwrap();
        let mut current_mm = current.mm.lock();
        let pa: usize = current_mm
            .page_table
            .translate(VirtAddr::from(buf as usize))
            .unwrap()
            .into();
        print!("{}", unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(pa as _, count))
        });
        Ok(count)
    }

    fn close(fd: usize) -> SyscallResult {
        todo!()
    }

    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult {
        todo!()
    }
}
