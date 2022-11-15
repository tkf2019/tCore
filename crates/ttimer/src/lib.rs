#![no_std]

mod test;

pub const MSEC_PER_SEC: usize = 1000;
pub const USEC_PER_SEC: usize = 1_000_000;
pub const USEC_PER_MSEC: usize = 1_000;
pub const NSEC_PER_SEC: usize = 1_000_000_000;
pub const NSEC_PER_MSEC: usize = 1_000_000;
pub const NSEC_PER_USEC: usize = 1_000;

/// Represents an elapsed time.
#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub struct TimeSpec {
    /// Number of whole seconds of elapsed time.
    pub tv_sec: usize,

    /// Number of nanoseconds of rest of elapsed time minus tv_sec.
    pub tv_nsec: usize,
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
