use ttimer::{ITimerVal, TimeSpec};

use crate::SyscallResult;

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
