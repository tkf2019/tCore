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
#![allow(non_camel_case_types)]

use numeric_enum_macro::numeric_enum;
use terrno::ErrNO;

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
    pub enum SyscallNO {
        OPENAT = 56,
        WRTIE = 64,
        EXIT = 93,
        GETPID = 172,
        CLONE = 220,
    }
}

pub type SyscallResult = Result<usize, ErrNO>;

pub trait SyscallProc {
    /// Terminate the calling process.
    fn exit(status: usize) -> !;

    /// Create a child process.
    fn clone(flags: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> SyscallResult;

    /// Execute the program referred to by pathname.
    ///
    /// This causes the program that is currently being run by the calling
    /// process to be replaced with a new program, with newly initialized
    /// stack, heap, and (initialized and uninitialized) data segments.
    fn execve(pathname: usize, argv: usize, envp: usize) -> SyscallResult;
}

pub trait SyscallFile {
    /// Open and possibly create a file
    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult;

    /// Close a file descriptor.
    fn close(fd: usize) -> SyscallResult;

    /// Write to a file descriptor.
    fn write(fd: usize, buf: *const u8, count: usize) -> SyscallResult;
}

pub trait SyscallDev {}

pub trait SyscallInfo {
    /// Get process identification, always successfully
    fn getpid() -> SyscallResult;
}

pub trait SyscallComm {}
