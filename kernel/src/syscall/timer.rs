use errno::Errno;
use syscall_interface::*;
use time_subsys::{TimeSpec, TimeVal, NSEC_PER_SEC};

use crate::{
    arch::{mm::VirtAddr, timer::get_time_sec_f64},
    read_user,
    task::{cpu, do_yield},
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

    fn nanosleep(req: usize, rem: usize) -> SyscallResult {
        let req_addr = VirtAddr::from(req);
        let mut req = TimeSpec::new(0.0);
        read_user!(cpu().curr.as_ref().unwrap().mm(), req_addr, req, TimeSpec)?;

        if req.tv_nsec >= NSEC_PER_SEC {
            return Err(Errno::EINVAL);
        }

        let end = get_time_sec_f64() + req.time_in_sec();
        while get_time_sec_f64() < end {
            unsafe { do_yield() };
        }

        if rem != 0 {
            let rem_addr = VirtAddr::from(rem);
            let rem = TimeSpec::new(0.0);
            write_user!(cpu().curr.as_ref().unwrap().mm(), rem_addr, rem, TimeSpec)?;
        }

        Ok(0)
    }
}
