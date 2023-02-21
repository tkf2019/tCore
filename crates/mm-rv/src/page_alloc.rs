use crate::{Page, PageRange};
use core::ops::Deref;

/// Represents a range of mapped virtual memory [`Page`]s.
///
/// These pages are allocated as the address space is created by kernel manager or else.
/// It must be initialized with a page table and share the ownership of the page table.
/// A single address space may have many [`AllocatedPageRange`]s to represent different memory
/// sections such as `.bss`, `.data` and etc.
pub struct AllocatedPageRange {
    /// Total continuous range of virtual pages.
    pages: PageRange,
}

impl AllocatedPageRange {
    /// Create mapped pages from an existing page table.
    pub fn new(start: Page, end: Page) -> Self {
        Self {
            pages: PageRange::new(start, end),
        }
    }
}

impl Deref for AllocatedPageRange {
    type Target = PageRange;
    fn deref(&self) -> &Self::Target {
        &self.pages
    }
}
