use crate::SyscallResult;

pub trait SyscallComm {
    /// Creates a pipe, a unidirectional data channel that can be used for
    /// interprocess communication.
    ///
    /// The array pipefd is used to return two file descriptors referring to
    /// the ends of the pipe. pipefd[0] refers to the read end of the pipe.
    /// pipefd[1] refers to the write end of the pipe.
    ///
    /// # Error
    /// - `EFAULT`: pipefd is not valid.
    /// - `EMFILE`: The per-process limit on the number of open file descriptor
    /// has been reached.
    fn pipe(pipefd: *const u32, flags: usize) -> SyscallResult;

    /// Used to change the action taken by a process on receipt of a specific signal.
    /// 
    /// # Argument
    /// - `signum`: specifies the signal and can be any valid signal except SIGKILL and SIGSTOP.
    /// - `act`: If act is non-NULL, the new action for signal signum is installed from act.
    /// If oldact is non-NULL, the previous action is saved in oldact.
    /// 
    /// # Error
    /// - `EFAULT`: act or oldact points to memory which is not a valid part of the process
    /// address space.
    /// - `EINVAL`: An invalid signal was specified.  This will also be generated if an attempt
    /// is made to change the action for SIGKILL or SIGSTOP, which cannot be caught or ignored.
    fn sigaction(signum: usize, act: usize, oldact: usize) -> SyscallResult;

    /// Sigprocmask() is used to fetch and/or change the signal mask of the calling 
    /// thread. The signal mask is the set of signals whose delivery is currently
    /// blocked for the caller (see also signal(7) for more details).
    /// 
    /// # Argument
    /// 
    /// - `how`: The behavior of the call is dependent on the value of how, as follows:
    ///   - `SIG_BLOCK`: The set of blocked signals is the union of the current set and 
    ///   the set argument.
    ///   - `SIG_UNBLOCK`: The signals in set are removed from the current set of 
    ///   blocked signals. It is permissible to attempt to unblock a signal which is 
    ///   not blocked.
    ///   - `SIG_SETMASK`: The set of blocked signals is set to the argument set.
    /// 
    /// - `set`: If oldset is non-NULL, the previous value of the signal mask is stored 
    /// in oldset. If set is NULL, then the signal mask is unchanged (i.e., how is 
    /// ignored), but the current value of the signal mask is nevertheless returned in
    /// oldset (if it is not NULL).
    /// 
    /// The use of sigprocmask() is unspecified in a multithreaded process; see
    /// pthread_sigmask(3).
    /// 
    /// # Error
    /// - `EFAULT`: The `set` or `oldset` argument points outside the process's allocated
    /// address space.
    /// - `EINVAL`: Either the value specified in how was invalid or the kernel does
    /// not support the size passed in sigsetsize.
    fn sigprocmask(how: usize, set: usize, oldset: usize, sigsetsize: usize) -> SyscallResult;

    /// Returns the `set` of signals that are pending for delivery to the calling thread
    /// (i.e., the signals which have been raised while blocked). The mask of pending
    /// signals is returned in set.
    /// 
    /// # Error
    /// - `EFAULT`: The `set` argument points outside the process's allocated
    /// address space.
    fn sigpending(set: usize) -> SyscallResult;
}
