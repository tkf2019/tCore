#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    /// An invalid page table entry.
    PageTableInvalid,

    /// Page has not been mapped to an frame yet.
    PageUnmapped,

    /// Failed to allocate a new frame: Internal Error
    FrameAllocFailed,

    /// Get frame out of the physical memory area
    FrameOutOfRange,

    /// Failed to resolve ELF
    /// - Wrong magic number
    /// - Unsupported architecture or XLEN
    ELFInvalid,

    /// Unsupported syscall
    SyscallUnsupported(usize),

    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    Interrupted,

    /// An error returned when an operation could not be completed because a
    /// call to `write` returned [`Ok(0)`].
    WriteZero,

    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    UnexpectedEof,
}

pub type KernelResult<T = ()> = Result<T, KernelError>;
