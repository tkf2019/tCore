use crate::arch::mm::PTEFlags;

bitflags::bitflags! {
    /// Flags in [`VMArea`].
    ///
    /// See linux `include/linux/mm.h`.
    pub struct VMFlags: u64 {
        const NONE = 0;
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC = 1 << 2;
        const SHARED = 1 << 3;

        /// General info on the segment.
        ///
        /// See [`MmapFlags::MAP_GROWSDOWN`].
        const GROWSDOWN = 1 << 8;

        /* Unstandard flags */

        /// Identical memory maps with no frame allocated
        const IDENTICAL = 1 << 62;

        /// User accessible 
        const USER = 1 << 63;
    }
}

impl From<VMFlags> for PTEFlags {
    fn from(value: VMFlags) -> Self {
        let mut flags = Self::empty();
        if value.contains(VMFlags::READ) {
            flags |= Self::READABLE;
        }
        if value.contains(VMFlags::WRITE) {
            flags |= Self::WRITABLE;
        }
        if value.contains(VMFlags::EXEC) {
            flags |= Self::EXECUTABLE;
        }
        if value.contains(VMFlags::USER) {
            flags |= Self::USER_ACCESSIBLE;
        }
        flags
    }
}

impl From<PTEFlags> for VMFlags {
    fn from(value: PTEFlags) -> Self {
        let mut flags = Self::empty();
        if value.contains(PTEFlags::READABLE) {
            flags |= Self::READ;
        }
        if value.contains(PTEFlags::WRITABLE) {
            flags |= Self::WRITE;
        }
        if value.contains(PTEFlags::EXECUTABLE) {
            flags |= Self::EXEC;
        }
        if value.contains(PTEFlags::USER_ACCESSIBLE) {
            flags |= Self::USER;
        }
        flags
    }
}

bitflags::bitflags! {
    /// Specified `prot` argument in [`SyscallProc::mmap`].
    pub struct MmapProt: usize {
        /// Pages may not be accessed.
        const PROT_NONE = 0;

        /// Pages may be read.
        const PROT_READ = 1 << 0;

        /// Pages may be written.
        const PROT_WRITE = 1 << 1;

        /// Pages may be executed.
        const PROT_EXEC = 1 << 2;
    }
}

impl From<MmapProt> for VMFlags {
    fn from(value: MmapProt) -> Self {
        let mut flags = Self::empty();
        if value.contains(MmapProt::PROT_READ) {
            flags |= Self::READ;
        }
        if value.contains(MmapProt::PROT_WRITE) {
            flags |= Self::WRITE;
        }
        if value.contains(MmapProt::PROT_EXEC) {
            flags |= Self::EXEC;
        }
        flags |= Self::USER;
        flags
    }
}

bitflags::bitflags! {
    /// Specified `flags` argument in [`SyscallProc::mmap`].
    pub struct MmapFlags: usize {
        /// Share this mapping. Updates to the mapping are visible to other
        /// processes mapping the same region, and (in the case of file-backed
        /// mappings) are carried through to the underlying file. (To precisely
        /// control when updates are carried through to the underlying file
        /// requires the use of msync(2).)
        const MAP_SHARED = 1 << 0;

        /// Create a private copy-on-write mapping. Updates to the mapping are
        /// not visible to other processes mapping the same file, and are not
        /// carried through to the underlying file. It is unspecified whether
        /// changes made to the file after the mmap() call are visible in the
        /// mapped region.
        const MAP_PRIVATE = 1 << 1;

        /// Don't interpret `addr` as a hint: place the mapping at exactly that
        /// address. `addr` must be suitably aligned: for most architectures a
        /// multiple of the page size is sufficient; however, some architectures
        /// may impose additional restrictions. If the memory region specified by
        /// `addr` and `length` overlaps pages of any existing mapping(s), then
        /// the overlapped part of the existing mapping(s) will be discarded.
        /// If the specified address cannot be used, mmap() will fail.
        const MAP_FIXED = 1 << 4;

        /// The mapping is not backed by any file; its contents are initialized
        /// to zero. The fd argument is ignored; however, some implementations
        /// require fd to be -1 if MAP_ANONYMOUS (or MAP_ANON) is specified, and
        /// portable applications should ensure this.
        /// The offset argument should be zero.
        const MAP_ANONYMOUS = 1 << 5;

        /// This flag is used for stacks. It indicates to the kernel virtual memory
        /// system that the mapping should extend downward in memory. The return address
        /// is one page lower than the memory area that is actually created in the
        /// process's virtual address space. Touching an address in the 'guard' page
        /// below the mapping will cause the mapping to grow by a page.
        /// This growth can be repeated until the mapping grows to within a page of the
        /// high end of the next lower mapping, at which point touching the 'guard' page
        /// will result in a `SIGSEGV` signal.
        const MAP_GROWSDOWN = 1 << 8;

        /// Mark the mapped region to be locked in the same way as mlock(2). This
        /// implementation will try to populate (prefault) the whole range but the mmap()
        /// call doesn't fail with ENOMEM if this fails. Therefore major faults might
        /// happen later on. So the semantic is not as strong as mlock(2). One should
        /// use mmap() plus mlock(2) when major faults are not acceptable after the
        /// initialization of the mapping. The `MAP_LOCKED` flag is ignored in older kernels.
        const MAP_LOCKED = 1 << 13;

        /// Do not reserve swap space for this mapping. When swap space is reserved,
        /// one has the guarantee that it is possible to modify the mapping.
        /// When swap space is not reserved one might get SIGSEGV upon a write if no
        /// physical memory is available.
        const MAP_NONRESERVE = 1 << 14;
    }
}
