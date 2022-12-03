use tsyscall::{SyscallFile, SyscallInfo, SyscallNO, SyscallProc, SyscallResult};

mod file;
mod info;
mod proc;
mod signal;

pub struct SyscallArgs(pub SyscallNO, pub [usize; 6]);

pub struct SyscallImpl;

pub fn syscall(args: SyscallArgs) -> SyscallResult {
    let id = args.0;
    let args = args.1;
    match id {
        SyscallNO::WRTIE => SyscallImpl::write(args[0], args[1] as *const u8, args[2]),
        SyscallNO::EXIT | SyscallNO::EXIT_GROUP => SyscallImpl::exit(args[0]),
        SyscallNO::SET_TID_ADDRESS => SyscallImpl::set_tid_address(args[0]),
        SyscallNO::GETPID => SyscallImpl::getpid(),
        SyscallNO::GETTID => SyscallImpl::gettid(),
        _ => {
            unimplemented!()
        }
    }
}
