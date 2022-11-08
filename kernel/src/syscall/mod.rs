use tsyscall::{ErrNO, SyscallNO, SyscallResult};

use crate::error::{KernelError, KernelResult};

use self::proc::getpid;

mod file;
mod proc;
mod signal;

pub struct SyscallArgs(SyscallNO, [usize; 6]);

pub fn syscall(args: SyscallArgs) -> SyscallResult {
    Ok(())
}
