use alloc::{borrow::ToOwned, string::String};
use core::{slice::from_raw_parts, str::from_utf8_unchecked};
use log::trace;
use tmm_rv::VirtAddr;
use tsyscall::*;

use crate::{print, println, task::current_task};

use super::SyscallImpl;

impl SyscallFile for SyscallImpl {
    fn write(fd: usize, buf: *const u8, count: usize) -> SyscallResult {
        let current = current_task().unwrap();
        let mut current_mm = current.mm.lock();
        let pa: usize = current_mm
            .page_table
            .translate(VirtAddr::from(buf as usize))
            .unwrap()
            .into();

        let s = unsafe { from_raw_parts(pa as *const u8, count) };
        print!("{} ", unsafe { from_utf8_unchecked(s) });
        Ok(count)
    }

    fn close(fd: usize) -> SyscallResult {
        todo!()
    }

    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult {
        todo!()
    }
}
