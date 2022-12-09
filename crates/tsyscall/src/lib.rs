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

use numeric_enum_macro::numeric_enum;
use terrno::Errno;
use ttimer::{ITimerType, ITimerVal, TimeSpec};

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
    #[allow(non_camel_case_types)]
    pub enum SyscallNO {
        OPENAT = 56,
        WRTIE = 64,
        EXIT = 93,
        EXIT_GROUP = 94,
        SET_TID_ADDRESS = 96,
        CLOCK_GET_TIME = 113,
        GETPID = 172,
        GETTID = 178,
        BRK = 214,
        MUNMAP = 215,
        CLONE = 220,
    }
}

pub type SyscallResult = Result<usize, Errno>;

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

    /// Get process identification, always successfully
    fn getpid() -> SyscallResult;

    /// Get thread identification, always successfully
    fn gettid() -> SyscallResult;

    /// Sets the clear_child_tid value for the calling thread to `tidptr`.
    ///
    /// # Return
    ///
    /// `set_tid_address()` always returns the caller's thread ID.
    fn set_tid_address(tidptr: usize) -> SyscallResult;

    /// 
    fn brk(brk: usize) -> SyscallResult;
    
    /// The munmap() system call deletes the mappings for the specified 
    /// address range, and causes further references to addresses within
    /// the range to generate invalid memory references. The region is
    /// also automatically unmapped when the process is terminated. On
    /// the other hand, closing the file descriptor does not unmap the region.
    /// 
    /// The address `addr` must be a multiple of the page size (but `length`
    /// need not be).  All pages containing a part of the indicated range
    /// are unmapped, and subsequent references to these pages will
    /// generate `SIGSEGV`.  It is not an error if the indicated range does
    /// not contain any mapped pages.
    /// 
    /// # Error
    /// - `EINVAL`:`addr` is not aligned to the page size. Or `len` is 0.
    /// - `ENOMEM`: unmapping a region in the middle of an existing mapping,
    /// since this results in two smaller mappings on either side of the
    /// region being unmapped.
    /// 
    /// # Reference
    /// - Linux: `https://elixir.bootlin.com/linux/latest/source/mm/mmap.c#L2757`.
    fn munmap(addr: usize, len: usize) -> SyscallResult;
}

pub trait SyscallFile {
    /// Open and possibly create a file
    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult;

    /// Close a file descriptor.
    fn close(fd: usize) -> SyscallResult;

    /// Writes to a file descriptor.
    ///
    /// # Error
    ///
    /// - `EFAULT`: buf is outside your accessible address space.
    /// - `EBADF`: fd is not a valid file descriptor or is not open for writing.
    /// - `EINVAL`: fd is attached to an object which is unsuitable for writing.
    fn write(fd: usize, buf: *const u8, count: usize) -> SyscallResult;
}

pub trait SyscallDev {}

pub trait SyscallTimer {
    /// Retrieves the time of specified clock `clockid`.
    ///
    /// # Error
    /// - `EFAULT`: tp points outside the accessible address space.
    fn clock_gettime(clockid: usize, tp: *mut TimeSpec) -> SyscallResult;

    /*
        These system calls provide access to interval timers, that is, timers that
        initially expire at some point in the future, and (optionally) at regular
        intervals after that. When a timer expires, a signal is generated for the
        calling process, and the timer is reset to the specified interval (if the
        interval is nonzero).
    */

    /// Places the current value of the timer specified by which in the buffer
    /// pointed to by `curr_value`.
    ///
    /// # Error
    /// - `EFAULT`: `curr_value` is not a valid pointer.
    /// - `EINVAL`: `which` is not one of [`ITimerType`]
    fn getitimer(which: usize, curr_value: *mut ITimerVal) -> SyscallResult;

    /// Arms or disarms the timer specified by `which`, by setting the timer to
    /// the value specified by `new_value`.
    ///
    /// If old_value is non-NULL, the buffer it points to is used to return the
    /// previous value of the timer (i.e., the same information that is returned
    /// by `getitimer()`).
    ///
    /// If either field in new_value.it_value is nonzero, then the timer is armed
    /// to initially expire at the specified time. If both fields in
    /// `new_value.it_value` are zero, then the timer is disarmed.
    ///
    /// # Error
    /// - `EFAULT`: `new_value` or `old_value` is not a valid pointer.
    /// - `EINVAL`: `which` is not one of [`ITimerType`]
    fn setitimer(
        which: usize,
        new_value: *const ITimerVal,
        old_value: *mut ITimerVal,
    ) -> SyscallResult;
}

pub trait SyscallComm {}
