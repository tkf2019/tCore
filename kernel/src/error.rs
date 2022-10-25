pub enum KernelError {
    Ok = 0,
    /* Page Table Errors */
    /// Encounters an invalid page table entry.
    PageTableInvalid,
    /* Frame Allocator Errors */
    FrameAllocFailed,
}

pub type KernelResult<T = ()> = Result<T, KernelError>;
