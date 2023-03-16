//! Syscall interfaces used in custom kernel.
//!
//! In Linux, making a system call involves transferring control from unprivileged
//! user mode to privileged kernel mode; the details of this transfer vary from
//! architecture to architecture. The libraries take care of collecting the
//! system-call arguments and, if necessary, arranging those arguments in the special
//! form necessary to make the system call.
//!
//! System calls can be divided into **5** categories mainly:
//! - Process control
//! - File management
//! - Device management
//! - Information maintainance
//! - Communication
//!
//! See [Linux Syscalls](https://man7.org/linux/man-pages/man2/syscalls.2.html) for linux
//! system call details.

#![no_std]
#![allow(unused)]
#![allow(non_camel_case_types)]

mod comm;
mod file;
mod io;
mod proc;
mod timer;

pub use comm::*;
use errno::Errno;
pub use file::*;
pub use io::*;
use numeric_enum_macro::numeric_enum;
pub use proc::*;
pub use timer::*;

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
    #[allow(non_camel_case_types)]
    pub enum SyscallNO {
        IOCTL = 29,
        MKDIRAT = 34,
        UNLINKAT = 35,
        LINKAT = 37,
        OPENAT = 56,
        CLOSE = 57,
        PIPE = 59,
        LSEEK = 62,
        READ = 63,
        WRTIE = 64,
        READV = 65,
        WRITEV = 66,
        PREAD = 67,
        EXIT = 93,
        EXIT_GROUP = 94,
        SET_TID_ADDRESS = 96,
        NANOSLEEP = 101,
        CLOCK_GET_TIME = 113,
        SIGACTION = 134,
        SIGPROCMASK = 135,
        SIGTIMEDWAIT = 137,
        SIGRETURN = 139,
        GET_TIME_OF_DAY = 169,
        GETPID = 172,
        GETTID = 178,
        BRK = 214,
        MUNMAP = 215,
        CLONE = 220,
        EXECVE = 221,
        MMAP = 222,
        MPROTECT = 226,
        WAIT4 = 260,
        PRLIMIT64 = 261,

        // UINTR
        UINTR_REGISTER_RECEIVER = 300,
        UINTR_REGISTER_SENDER = 301,
        UINTR_CREATE_FD = 302,
    }
}

pub type SyscallResult = Result<usize, Errno>;
