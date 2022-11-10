use tsyscall::{SyscallNO, SyscallResult};

mod file;
mod proc;
mod signal;

pub struct SyscallArgs(pub SyscallNO, pub [usize; 6]);

pub fn syscall(args: SyscallArgs) -> SyscallResult {
    Ok(())
}
