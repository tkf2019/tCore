use _core::mem::size_of;
use alloc::{vec, vec::Vec};
use bitflags::*;

use crate::{
    frame_alloc, AllocatedFrames, Frame, Page, PhysAddr, VirtAddr, PPN_MASK_SV39, PPN_OFFSET_SV39,
};

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
        Self::empty()
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
#[derive(Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Create a new page table entry from physical address.
    pub fn new(addr: PhysAddr) -> Self {
        unsafe { PageTableEntry(*(addr.value() as *const u64)) }
    }

    /// Returns an uninit page table entry with no flags and ppns.
    pub fn zero() -> Self {
        PageTableEntry(0)
    }

    /// Returns the flags of this [`PageTableEntry`]
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    /// Returns the physical frame pointed by the `PPN` segment.
    ///
    /// If the page table entry is not valid, it returns to `None`.
    pub fn frame(&self) -> Frame {
        Frame::floor(PhysAddr::new_canonical((self.0 << 2) as usize))
    }

    /// Set flags
    #[inline]
    pub fn set_flags(&mut self, flags: PTEFlags) {
        self.0 = self.0 & PPN_MASK_SV39 as u64 | flags.bits();
    }

    /// Set physical frame number
    #[inline]
    pub fn set_ppn(&mut self, frame: &Frame) {
        self.0 = ((frame.number() as u64) << PPN_OFFSET_SV39) & PPN_MASK_SV39 as u64
            | self.flags().bits();
    }

    /// Get the physical address from start address of the frame and index of
    /// this [`PageTableEntry`].
    pub fn from_index(frame: &Frame, index: usize) -> PhysAddr {
        PhysAddr::new_canonical(frame.start_address().value() + index * size_of::<PageTableEntry>())
    }

    /// `Unsafe` writes the page table entry to the address.
    pub fn write(&self, addr: PhysAddr) {
        unsafe { *(addr.value() as *mut PageTableEntry) = self.clone() };
    }
}

bitflags! {
    /// Page table walker flag bits.
    pub struct PTWalkerFlags:u8 {
        /// Create new page table entries while walking down the page table.
        const CREAT = 1 << 0;
    }
}

/// Page table in SV39
#[derive(Debug)]
pub struct PageTable {
    /// Root frame pointed by `satp`
    root: Frame,

    /// Allocated frames of this [`PageTable`].
    /// New page table entries will be created by map requests, so available physical frames need
    /// to be allocated when walking down the 3-level page table in SV39.
    frames: Vec<AllocatedFrames>,
}

impl PageTable {
    /// Create a page table with a newly allocated root frame.
    pub fn new() -> Result<Self, &'static str> {
        let root_frame = AllocatedFrames::new(1)?;
        Ok(Self {
            // No iteration after a successful allocation, thus do `unwrap()` freely.
            root: root_frame.start(),
            frames: vec![root_frame],
        })
    }

    /// Walk down this [`PageTable`], The virtual page number is given.
    /// In SV39, `vpn` is splitted into 3 indexes, 9 bits each, which is to locate the
    /// [`PageTableEntry`] among 512 entries in a 4KB page table frame.
    /// We will allocate a new frame to create new entries and set the valid bit if the
    /// `CREAT` bit is set in flags.
    ///
    /// This function cannot be used outside the page table. It checks the `valid` bit of the leaf.
    fn walk(
        &mut self,
        page: Page,
        flags: PTWalkerFlags,
    ) -> Result<(PhysAddr, PageTableEntry), &'static str> {
        let indexes = page.split_vpn();
        let mut link = self.root;
        let mut result: Option<(PhysAddr, PageTableEntry)> = None;
        for (j, index) in indexes.iter().enumerate() {
            let pa = PageTableEntry::from_index(&link, *index);
            let mut entry = &mut PageTableEntry::new(pa);
            // No existing entry, create a new one.
            if !entry.flags().is_valid() {
                if flags.intersects(PTWalkerFlags::CREAT) && j < 2 {
                    let new_frame = AllocatedFrames::new(1)?;

                    // Write new valid entry to the target frame.
                    entry.set_flags(PTEFlags::VALID);
                    entry.set_ppn(&new_frame.start());
                    entry.write(pa);

                    // Delegate the ownership to this page table
                    self.frames.push(new_frame);
                } else {
                    return Err("Encounter an invalid page table entry.");
                }
            }
            // Reach the leaf page table frame.
            result = Some((pa, entry.clone()));
            link = entry.frame();
        }
        Ok(result.unwrap())
    }

    /// Virtual page will be mapped to physical frame. Caller must guarantee that the frame
    /// has been allocated and will not be used again by the `PageTableWalker`.
    pub fn map(&mut self, page: Page, frame: Frame, flags: PTEFlags) -> Result<(), &'static str> {
        let (pa, mut pte) = self.walk(page, PTWalkerFlags::CREAT)?;
        pte.set_flags(flags);
        pte.set_ppn(&frame);
        pte.write(pa);
        Ok(())
    }

    /// Clears the page table entry found by the page.
    pub fn unmap(&mut self, page: Page) -> Result<(), &'static str> {
        let (pa, mut pte) = self.walk(page, PTWalkerFlags::empty())?;
        pte = PageTableEntry::zero();
        pte.write(pa);
        Ok(())
    }

    /// Translate virtual address into physical address.
    pub fn translate(&mut self, va: VirtAddr) -> Result<PhysAddr, &'static str> {
        self.walk(Page::floor(va), PTWalkerFlags::empty())
            .map(|(_, pte)| {
                let offset = va.page_offset();
                let pa = pte.frame().start_address();
                pa + offset
            })
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self {
            root: Frame::ceil(PhysAddr::zero()),
            frames: Vec::new(),
        }
    }
}
