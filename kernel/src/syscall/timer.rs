use terrno::Errno;
use tmm_rv::VirtAddr;
use tsyscall::*;
use ttimer::TimeSpec;

use crate::{task::manager::current_task, timer::get_time_sec_f64};

use super::SyscallImpl;

impl SyscallTimer for SyscallImpl {
    fn clock_gettime(_clockid: usize, tp: *mut TimeSpec) -> SyscallResult {
        let current = current_task().unwrap();
        let mut current_mm = current.mm.lock();

        // Get time specification from user address space.
        current_mm
            .alloc_write_type(
                VirtAddr::from(tp as usize),
                &TimeSpec::new(get_time_sec_f64()),
            )
            .map_err(|_| Errno::EFAULT)?;

        Ok(0)
    }

    fn getitimer(which: usize, curr_value: *mut ttimer::ITimerVal) -> SyscallResult {
        Ok(0)
    }

    fn setitimer(
        which: usize,
        new_value: *const ttimer::ITimerVal,
        old_value: *mut ttimer::ITimerVal,
    ) -> SyscallResult {
        Ok(0)
    }
}
