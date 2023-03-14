use log::trace;
use syscall_interface::{
    IoVec, SyscallComm, SyscallFile, SyscallIO, SyscallNO, SyscallProc, SyscallResult, SyscallTimer,
};

mod comm;
mod file;
mod io;
mod proc;
mod timer;

#[derive(Debug)]
pub struct SyscallArgs(pub SyscallNO, pub [usize; 6]);

pub struct SyscallImpl;

pub fn syscall(args: SyscallArgs) -> SyscallResult {
    trace!("[U] SYSCALL {:X?}", args);
    let id = args.0;
    let args = args.1;
    match id {
        SyscallNO::IOCTL => SyscallImpl::ioctl(args[0], args[1], args[2] as *const usize),
        SyscallNO::UNLINKAT => SyscallImpl::unlinkat(args[0], args[1] as *const u8, args[2]),
        SyscallNO::OPENAT => SyscallImpl::openat(args[0], args[1] as *const u8, args[2], args[3]),
        SyscallNO::CLOSE => SyscallImpl::close(args[0]),
        SyscallNO::PIPE => SyscallImpl::pipe(args[0] as *const u32, args[1]),
        SyscallNO::LSEEK => SyscallImpl::lseek(args[0], args[1], args[2]),
        SyscallNO::READ => SyscallImpl::read(args[0], args[1] as *mut u8, args[2]),
        SyscallNO::WRTIE => SyscallImpl::write(args[0], args[1] as *const u8, args[2]),
        SyscallNO::READV => SyscallImpl::readv(args[0], args[1] as *const IoVec, args[2]),
        SyscallNO::WRITEV => SyscallImpl::writev(args[0], args[1] as *const IoVec, args[2]),
        SyscallNO::EXIT | SyscallNO::EXIT_GROUP => SyscallImpl::exit(args[0]),
        SyscallNO::SET_TID_ADDRESS => SyscallImpl::set_tid_address(args[0]),
        SyscallNO::CLOCK_GET_TIME => SyscallImpl::clock_gettime(args[0], args[1]),
        SyscallNO::SIGACTION => SyscallImpl::sigaction(args[0], args[1], args[2]),
        SyscallNO::SIGPROCMASK => SyscallImpl::sigprocmask(args[0], args[1], args[2], args[3]),
        SyscallNO::SIGTIMEDWAIT => SyscallImpl::sigtimedwait(args[0], args[1], args[2]),
        SyscallNO::GET_TIME_OF_DAY => SyscallImpl::gettimeofday(args[0]),
        SyscallNO::GETPID => SyscallImpl::getpid(),
        SyscallNO::GETTID => SyscallImpl::gettid(),
        SyscallNO::BRK => SyscallImpl::brk(args[0]),
        SyscallNO::MUNMAP => SyscallImpl::munmap(args[0], args[1]),
        SyscallNO::CLONE => SyscallImpl::clone(args[0], args[1], args[2], args[3], args[4]),
        SyscallNO::WAIT4 => SyscallImpl::wait4(args[0] as isize, args[1], args[2], args[3]),
        SyscallNO::PRLIMIT64 => SyscallImpl::prlimit64(args[0] as isize, args[1] as i32, args[2], args[3]),
        SyscallNO::MMAP => SyscallImpl::mmap(args[0], args[1], args[2], args[3], args[4], args[5]),
        SyscallNO::MPROTECT => SyscallImpl::mprotect(args[0], args[1], args[2]),
        _ => {
            unimplemented!("{:?}", id)
        }
    }
}
