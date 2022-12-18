use terrno::Errno;
use tmm_rv::VirtAddr;
use tsyscall::*;
use ttimer::{TimeSpec, TimeVal};

use crate::{task::manager::current_task, timer::get_time_sec_f64};

use super::SyscallImpl;

impl SyscallTimer for SyscallImpl {
    fn clock_gettime(_clockid: usize, tp: usize) -> SyscallResult {
        let current = current_task().unwrap();
        let mut mm = current.mm.lock();

        // Get time specification from user address space.
        mm.alloc_write_type(VirtAddr::from(tp), &TimeSpec::new(get_time_sec_f64()))
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
        let current = current_task().unwrap();
        let mut mm = current.mm.lock();

        // Get time specification from user address space.
        mm.alloc_write_type(VirtAddr::from(tv), &TimeVal::new(get_time_sec_f64()))
            .map_err(|_| Errno::EFAULT)?;

        Ok(0)
    }
}
