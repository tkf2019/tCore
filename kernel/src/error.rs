pub enum KernelError {
    Ok = 0,
    /* Page Table Errors */
    /// Encounters an invalid page table entry.
    PageTableInvalid,
    /// Page has not been mapped to an frame yet.
    PageUnmapped,
    /* Frame Allocator Errors */
    FrameAllocFailed,
}

pub type KernelResult<T = ()> = Result<T, KernelError>;
