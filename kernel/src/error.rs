pub enum KernelError {
    Ok = 0,
    /// An invalid page table entry.
    PageTableInvalid,
    /// Page has not been mapped to an frame yet.
    PageUnmapped,
    /// Failed to allocate a new frame: Internal Error
    FrameAllocFailed,
    /// Get frame out of the physical memory area
    FrameOutOfRange,
    /// Failed to resolve ELF
    /// - Wrong magic number: 
    ELFInvalid,
}

pub type KernelResult<T = ()> = Result<T, KernelError>;
