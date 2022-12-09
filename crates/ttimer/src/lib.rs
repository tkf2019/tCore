#![no_std]

mod config;
mod spec;
mod test;

pub use config::*;
use numeric_enum_macro::numeric_enum;
pub use spec::*;

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    #[allow(non_camel_case_types)]
    pub enum ClockType {
        REALTIME = 0,
        MONOTONIC = 1,
        PROCESS_CPUTIME_ID = 2,
        THREAD_CPUTIME_ID = 3,
    }
}

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum CPUClockType {
        PROF = 0,
        VIRT = 1,
        SCHED = 2,
        FD = 3,
    }
}

/// CPU clock identification.
///
/// Bit fields within a clock id:
/// - [31:3] hold either a pid or a file descriptor
/// - [2] indicates whether a cpu clock refers to a thread or a process
/// - [1:0] give [`CPUClockType`]
///
/// A clock id is invalid if bits 2, 1 and 0 are all set.
pub struct ClockID(i32);

impl ClockID {
    ///  Creates a new clock id.
    pub fn new(clock: usize) -> Self {
        Self(clock as i32)
    }

    /// Creates a new clock id from pid.
    pub fn new_proc(pid: usize, type_: CPUClockType) -> Self {
        Self((((!pid) << 3) | usize::from(type_)) as i32)
    }

    /// Creates a new clock id from tid.
    pub fn new_thread(tid: usize, type_: CPUClockType) -> Self {
        Self((((!tid) << 3) | usize::from(type_) | 4) as i32)
    }

    /// Gets pid from a clock id.
    pub fn get_pid(&self) -> usize {
        !(self.0 as usize >> 3)
    }

    /// Gets clock type.
    pub fn get_type(&self) -> CPUClockType {
        CPUClockType::try_from(self.0 as usize & 3).unwrap()
    }

    /// Returns whether it refers to a thread or a process.
    pub fn is_thread(&self) -> bool {
        (self.0 as usize & 4) != 0
    }

    /// Returns whether it refers to a process.
    pub fn is_proc(&self) -> bool {
        (self.0 as usize & 4) == 0
    }
}

/// System clock abstraction for different clocks.  
///
/// See more details in Linux `struct k_clock`.
pub trait Clock {
    /// Get the resolution of a global clock with the given identification.
    fn clock_getres(which: ClockID, tp: &mut TimeSpec) -> usize;

    /// Get a global clock with the given identificaton.
    fn clock_get(which: ClockID, tp: &mut TimeSpec) -> usize;
}
