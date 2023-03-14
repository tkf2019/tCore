use errno::Errno;
use syscall_interface::*;

use crate::{
    arch::mm::{VirtAddr, LOW_MAX_VA},
    config::USER_STACK_SIZE,
    read_user, write_user,
};

use super::*;

pub fn do_prlimit(resource: i32, new_limit: usize, old_limit: usize) -> SyscallResult {
    let curr = cpu().curr.as_ref().unwrap();
    let mut old_rlimit = Rlimit::default();
    let mut new_rlimit = Rlimit::default();

    if new_limit != 0 {
        read_user!(curr.mm(), VirtAddr::from(new_limit), new_rlimit, Rlimit)?;
        if new_rlimit.rlim_cur > new_rlimit.rlim_max {
            return Err(Errno::EINVAL);
        }
    }

    match resource {
        RLIMIT_STACK => {
            old_rlimit = Rlimit {
                rlim_cur: USER_STACK_SIZE as u64,
                rlim_max: USER_STACK_SIZE as u64,
            };
        }
        RLIMIT_NOFILE => {
            let limit = curr.files().get_limit() as u64;
            old_rlimit = Rlimit {
                rlim_cur: limit,
                rlim_max: limit,
            };
            if new_limit != 0 {
                curr.files().set_limit(new_rlimit.rlim_cur as usize);
            }
        }
        RLIMIT_AS => {
            old_rlimit = Rlimit {
                rlim_cur: (LOW_MAX_VA + 1) as u64,
                rlim_max: (LOW_MAX_VA + 1) as u64,
            }
        }
        _ => {
            return Err(Errno::EINVAL);
        }
    }

    if old_limit != 0 {
        write_user!(curr.mm(), VirtAddr::from(old_limit), old_rlimit, Rlimit)?;
    }

    Ok(0)
}
