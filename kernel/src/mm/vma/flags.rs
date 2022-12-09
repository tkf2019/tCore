use tmm_rv::PTEFlags;

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

        /// Not standard.
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
