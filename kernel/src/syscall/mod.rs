use tsyscall::{SyscallFile, SyscallNO, SyscallProc, SyscallResult};

use crate::error::{KernelError, KernelResult};

mod file;
mod proc;
mod signal;

pub struct SyscallArgs(pub SyscallNO, pub [usize; 6]);

pub struct SyscallImpl;

pub fn syscall(args: SyscallArgs) -> SyscallResult {
    let id = args.0;
    let args = args.1;
    match id {
        SyscallNO::WRTIE => SyscallImpl::write(args[0], args[1] as *mut u8, args[2]),
        SyscallNO::EXIT => SyscallImpl::exit(args[0]),
        _ => {
            unimplemented!()
        }
    }
}
