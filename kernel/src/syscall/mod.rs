use log::trace;
use tsyscall::{SyscallFile, SyscallNO, SyscallProc, SyscallResult, SyscallTimer};
use ttimer::TimeSpec;

mod file;
mod proc;
mod signal;
mod timer;

#[derive(Debug)]
pub struct SyscallArgs(pub SyscallNO, pub [usize; 6]);

pub struct SyscallImpl;

pub fn syscall(args: SyscallArgs) -> SyscallResult {
    trace!("[U] Syscall {:X?}", args);
    let id = args.0;
    let args = args.1;
    match id {
        SyscallNO::WRTIE => SyscallImpl::write(args[0], args[1] as *const u8, args[2]),
        SyscallNO::EXIT | SyscallNO::EXIT_GROUP => SyscallImpl::exit(args[0]),
        SyscallNO::SET_TID_ADDRESS => SyscallImpl::set_tid_address(args[0]),
        SyscallNO::GETPID => SyscallImpl::getpid(),
        SyscallNO::GETTID => SyscallImpl::gettid(),
        SyscallNO::CLOCK_GET_TIME => SyscallImpl::clock_gettime(args[0], args[1] as *mut TimeSpec),
        SyscallNO::BRK => SyscallImpl::brk(args[0]),
        SyscallNO::MUNMAP => SyscallImpl::munmap(args[0], args[1]),
        SyscallNO::MMAP => SyscallImpl::mmap(args[0], args[1], args[2], args[3], args[4], args[5]),
        SyscallNO::OPENAT => SyscallImpl::openat(args[0], args[1] as *const u8, args[2], args[3]),
        _ => {
            unimplemented!("{:?}", id)
        }
    }
}
