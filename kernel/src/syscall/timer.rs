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
        let time = TimeSpec::new(get_time_sec_f64());
        write_user!(
            cpu().curr.as_ref().unwrap().mm(),
            VirtAddr::from(tp),
            time,
            TimeSpec
        )?;
        Ok(0)
    }

    fn gettimeofday(tv: usize) -> SyscallResult {
        let time = TimeVal::new(get_time_sec_f64());
        write_user!(
            cpu().curr.as_ref().unwrap().mm(),
            VirtAddr::from(tv),
            time,
            TimeVal
        )?;
        Ok(0)
    }
}
