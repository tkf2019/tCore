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

mod file;
mod proc;
mod timer;

pub use file::*;
use numeric_enum_macro::numeric_enum;
pub use proc::*;
use terrno::Errno;
pub use timer::*;

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
    #[allow(non_camel_case_types)]
    pub enum SyscallNO {
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
        GET_TIME_OF_DAY = 169,
        GETPID = 172,
        GETTID = 178,
        BRK = 214,
        MUNMAP = 215,
        CLONE = 220,
        MMAP = 222,
    }
}

pub type SyscallResult = Result<usize, Errno>;

pub trait SyscallDev {}

pub trait SyscallComm {}
