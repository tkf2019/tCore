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
}
