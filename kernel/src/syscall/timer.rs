use errno::Errno;
use syscall_interface::*;
use time_subsys::{TimeSpec, TimeVal};

use crate::{
    arch::{mm::VirtAddr, timer::get_time_sec_f64},
    task::cpu,
    write_user,
};

use super::SyscallImpl;

impl SyscallTimer for SyscallImpl {
    fn clock_gettime(_clockid: usize, tp: usize) -> SyscallResult {
        write_user!(
            cpu().curr.as_ref().unwrap().mm(),
            VirtAddr::from(tp),
            TimeSpec::new(get_time_sec_f64()),
            TimeSpec
        )?;
        Ok(0)
    }

    fn gettimeofday(tv: usize) -> SyscallResult {
        write_user!(
            cpu().curr.as_ref().unwrap().mm(),
            VirtAddr::from(tv),
            TimeVal::new(get_time_sec_f64()),
            TimeVal
        )?;
        Ok(0)
    }
}
