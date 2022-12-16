use core::ops::{Add, AddAssign};

use numeric_enum_macro::numeric_enum;

use crate::{config::NSEC_PER_SEC, USEC_PER_SEC};

/// Represents an elapsed time.
#[repr(C)]
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct TimeSpec {
    /// Number of whole seconds of elapsed time.
    pub tv_sec: usize,

    /// Number of nanoseconds of rest of elapsed time minus tv_sec.
    pub tv_nsec: usize,
}

impl TimeSpec {
    /// Create a new time specification.
    pub fn new(sec: f64) -> Self {
        Self {
            tv_sec: sec as usize,
            tv_nsec: ((sec - sec as usize as f64) * NSEC_PER_SEC as f64) as usize,
        }
    }

    /// Returns time in seconds.
    pub fn time_in_sec(&self) -> f64 {
        self.tv_sec as f64 + self.tv_nsec as f64 / NSEC_PER_SEC as f64
    }
}

impl Add for TimeSpec {
    type Output = TimeSpec;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new_ts = Self {
            tv_sec: self.tv_sec + rhs.tv_sec,
            tv_nsec: self.tv_nsec + rhs.tv_nsec,
        };
        if new_ts.tv_nsec >= NSEC_PER_SEC {
            new_ts.tv_sec += 1;
            new_ts.tv_nsec -= NSEC_PER_SEC;
        }
        new_ts
    }
}

impl AddAssign for TimeSpec {
    fn add_assign(&mut self, rhs: Self) {
        let mut new_ts = Self {
            tv_sec: self.tv_sec + rhs.tv_sec,
            tv_nsec: self.tv_nsec + rhs.tv_nsec,
        };
        if new_ts.tv_nsec >= NSEC_PER_SEC {
            new_ts.tv_sec += 1;
            new_ts.tv_nsec -= NSEC_PER_SEC;
        }
        *self = new_ts;
    }
}

/// Represents an elapsed time.
#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub struct TimeVal {
    /// Number of whole seconds of elapsed time.
    pub tv_sec: usize,

    /// Number of microseconds of rest of elapsed time minus tv_sec.
    pub tv_usec: usize,
}

impl TimeVal {
    /// Create a new time specification.
    pub fn new(sec: f64) -> Self {
        Self {
            tv_sec: sec as usize,
            tv_usec: ((sec - sec as usize as f64) * USEC_PER_SEC as f64) as usize,
        }
    }

    /// Returns time in seconds.
    pub fn time_in_sec(&self) -> f64 {
        self.tv_sec as f64 + self.tv_usec as f64 / USEC_PER_SEC as f64
    }
}

/// Syscall `times()` stores current process times in this struct.
#[repr(C)]
#[derive(Debug)]
pub struct TMS {
    /// User time
    pub utime: usize,

    /// System time
    pub stime: usize,

    /// User time of children
    pub cutime: usize,

    /// System time of children
    pub cstime: usize,
}

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug)]
    pub enum ITimerType {
        /// This timer counts down in real (i.e., wall clock) time.
        /// At each expiration, a SIGALRM signal is generated.
        REAL = 0,

        /// This timer counts down against the user-mode CPU time
        /// consumed by the process. (The measurement includes CPU
        /// time consumed by all threads in the process.)  At each
        /// expiration, a `SIGVTALRM` signal is generated.
        VIRTUAL = 1,

        /// This timer counts down against the total (i.e., both user
        /// and system) CPU time consumed by the process.  (The
        /// measurement includes CPU time consumed by all threads in
        /// the process.)  At each expiration, a `SIGPROF` signal is generated.
        /// 
        /// In conjunction with [`Self::VIRTUAL`], this timer can be used
        /// to profile user and system CPU time consumed by the process.
        PROF = 2,
    }
}

/// Syscall `getitimer()` and `setitimer` handle user timer with this struct.
#[repr(C)]
#[derive(Debug)]
pub struct ITimerVal {
    /// Interval for periodic timer.
    ///
    /// If both fields of it_interval are zero, then this is a single-shot timer
    /// (i.e., it expires just once).
    pub it_interval: TimeVal,

    /// Time until next expiration.
    ///
    /// This value changes as the timer counts down, and will be reset to
    /// `it_interval` when the timer expires. If both fields of it_value
    /// are zero, then this timer is currently disarmed (inactive).
    pub it_value: TimeVal,
}
