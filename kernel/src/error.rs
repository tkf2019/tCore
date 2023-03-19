#![allow(unused)]

use errno::Errno;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    /// Unimplemented functions
    Unimplemented,

    /// Invalid arguments
    InvalidArgs,

    /// A warpper for errno
    Errno(Errno),

    /// Unsupported syscall
    SyscallUnsupported(usize),

    /// An invalid page table entry.
    PageTableInvalid,

    /// Failed to allocate a new frame: Internal Error
    FrameAllocFailed,

    /// Get frame out of the physical memory area
    FrameOutOfRange,

    /// Failed to get the frame
    FrameNotFound,

    /// Failed to resolve ELF
    /// - Wrong magic number
    /// - Unsupported architecture or XLEN
    ELFInvalidHeader,

    ELFInvalidSegment,

    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    IOInterrupted,

    /// An error returned when an operation could not be completed because a
    /// call to `write` returned [`Ok(0)`].
    IOWriteZero,

    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    IOUnexpectedEof,

    /// FD out of bound or removed.
    FDNotFound,

    /// FD exceeds limit
    FDOutOfBound,

    /// Page has not been mapped.
    PageUnmapped,

    /// Page fault cannot be handled.
    FatalPageFault,

    /// Cannot find the virtual memory area.
    VMANotFound,

    /// VMA failed to read ot write file.
    VMAFailedIO,

    /// Run out of free memory
    VMAAllocFailed,
}

pub type KernelResult<T = ()> = Result<T, KernelError>;

impl From<KernelError> for Errno {
    fn from(value: KernelError) -> Self {
        match value {
            KernelError::Errno(errno) => errno.clone(),
            KernelError::PageTableInvalid => Errno::EFAULT,
            KernelError::InvalidArgs => Errno::EINVAL,
            KernelError::FDNotFound => Errno::EBADF,
            KernelError::VMANotFound | KernelError::VMAAllocFailed => Errno::ENOMEM,
            KernelError::VMAFailedIO => Errno::EACCES,
            
            // TODO
            _ => Errno::EINVAL,
        }
    }
}
