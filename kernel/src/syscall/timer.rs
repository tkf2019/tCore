use errno::Errno;
use syscall_interface::*;
use time_subsys::{TimeSpec, TimeVal};

use crate::{
    arch::{mm::VirtAddr, timer::get_time_sec_f64},
    task::curr_task,
};

use super::SyscallImpl;

impl SyscallTimer for SyscallImpl {
    fn clock_gettime(_clockid: usize, tp: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();

        // Get time specification from user address space.
        curr_mm
            .alloc_write_type(tp.into(), &TimeSpec::new(get_time_sec_f64()))
            .map_err(|_| Errno::EFAULT)?;

        Ok(0)
    }

    fn getitimer(which: usize, curr_value: usize) -> SyscallResult {
        Ok(0)
    }

    fn setitimer(which: usize, new_value: usize, old_value: usize) -> SyscallResult {
        Ok(0)
    }

    fn gettimeofday(tv: usize) -> SyscallResult {
        let curr = curr_task().unwrap();
        let mut curr_mm = curr.mm.lock();

        // Get time specification from user address space.
        curr_mm
            .alloc_write_type(VirtAddr::from(tv), &TimeVal::new(get_time_sec_f64()))
            .map_err(|_| Errno::EFAULT)?;

        Ok(0)
    }
}
