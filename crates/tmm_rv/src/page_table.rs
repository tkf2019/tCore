use _core::mem::size_of;
use alloc::{vec, vec::Vec};
use bitflags::*;

use crate::{frame::AllocatedFrames, Frame, PhysAddr, PPN_MASK_SV39};

bitflags! {
    /// Page table entry flag bits in SV39
    pub struct PTEFlags: u64 {
        /// Iff set, the entry is valid.
        const VALID = 1 << 0;

        /// If set, reads to this page is allowed.
        const READABLE = 1 << 1;

        /// If set, writes to this page is allowed.
        const WRITABLE = 1 << 2;

        /// If set, bytes in this page can be executed as programs.
        /// If not set, this page is only used for data storage.
        const EXECUTABLE = 1 << 3;

        /// If set, this page is accessible in user space (U-mode).
        /// If not set, only kernel space (S-mode) can access this page.
        const USER_ACCESSIBLE = 1 << 4;

        /// If set, this page is accessible in all privileges.
        const GLOBAL = 1 << 5;

        /// If the entry is recently accessed.
        const ACCESSED = 1 << 6;

        /// If the entry is recently modified.
        /// Must be zero in page directory.
        const DIRTY = 1 << 7;
    }
}

impl PTEFlags {
    /// Returns a new, all-zero [`PTEFlags`] with no bits set.
    ///
    /// This is a `const` version of `Default::default`
    pub const fn zero() -> PTEFlags {
        PTEFlags::from_bits_truncate(0)
    }

    /// Returns true if the page is valid.
    pub const fn is_valid(&self) -> bool {
        self.intersects(PTEFlags::VALID)
    }

    /// Returns true if the page is writable.
    pub const fn is_writable(&self) -> bool {
        self.intersects(PTEFlags::WRITABLE)
    }

    /// Returns true if the page is readable.
    pub const fn is_readable(&self) -> bool {
        self.intersects(PTEFlags::READABLE)
    }

    /// Returns true if the page is executable.
    pub const fn is_executable(&self) -> bool {
        self.intersects(PTEFlags::EXECUTABLE)
    }

    /// Returns true if the page table entry points to next level of page table.
    pub const fn is_pointer(&self) -> bool {
        self.is_valid() & !self.is_executable() & !self.is_writable() & !self.is_readable()
    }
}

impl Default for PTEFlags {
    fn default() -> Self {
        Self::zero()
    }
}

/// Page table entry in SV39
///
/// The designation of bits in each page table entry is as such:
/// - 63:54 -> Reserved (wired to zero)
/// - 53:28 -> PPN\[2\]
/// - 27:19 -> PPN\[1\]
/// - 18:10 -> PPN\[0\]
/// - 9:8   -> Reserved for supervisor sofware
/// - 7:0   -> Flags
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn new(frame: Frame, index: usize) -> Self {
        let pa: usize = frame.start_address().value() + index * size_of::<PageTableEntry>();
        unsafe { PageTableEntry(*(pa as *const u64)) }
    }

    /// Returns the flags of this [`PageTableEntry`]
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    /// Returns the physical frame pointed by the `PPN` segment.
    ///
    /// If the page table entry is not valid, it returns to `None`.
    pub fn frame(&self) -> Option<Frame> {
        if self.flags().is_valid() {
            Some(Frame::ceil(PhysAddr::new_canonical((self.0 << 2) as usize)))
        } else {
            None
        }
    }

    /// Set flags of this [`PageTableEntry`]
    pub fn set_flags(&mut self, flags: PTEFlags) {
        self.0 = self.0 & PPN_MASK_SV39 as u64 | flags.bits();
    }
}

/// Page table in SV39
#[derive(Debug)]
pub struct PageTable {
    /// Root frame pointed by `satp`
    root: Frame,

    /// Allocated frames of this [`PageTable`].
    /// New page table entries will be created by map requests, so available physical frames need
    /// to be allocated when walking through the 3-level page table in SV39.
    frames: Vec<AllocatedFrames>,
}

impl PageTable {
    pub fn new() -> Option<Self> {
        if let Some(root_frame) = AllocatedFrames::new(1) {
            Some(Self {
                // No iteration after a successful allocation, thus do `unwrap()` freely.
                root: root_frame.start().unwrap(),
                frames: vec![root_frame],
            })
        } else {
            None
        }
    }
}
