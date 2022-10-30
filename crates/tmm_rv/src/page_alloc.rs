use crate::{AllocatedFrames, Frame, PTEFlags, Page, PageRange, PageTable, VirtAddr};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::ops::{Deref, DerefMut};
use spin::Mutex;

/// Represents a range of mapped virtual memory [`Page`]s.
///
/// These pages are allocated as the address space is created by kernel manager or else.
/// It must be initialized with a page table and share the ownership of the page table.
/// A single address space may have many [`AllocatedPages`]s to represent different memory
/// sections such as `.bss`, `.data` and etc.
pub struct AllocatedPages {
    /// Total continuous range of virtual pages.
    pub pages: PageRange,
}

impl AllocatedPages {
    /// Create mapped pages from an existing page table.
    pub fn new(start: Page, end: Page) -> Self {
        Self {
            pages: PageRange::new(start, end),
        }
    }

    /// Splits this [`AllocatedPages`] into two separate objects:
    /// - `[beginning : at_page - 1]`
    /// - `[at_page : end]`
    ///
    /// This function follows the behavior of [`core::slice::split_at()`],
    /// thus, either one of the returned `AllocatedPages` objects may be empty.
    /// - If `at_page == self.start`, the left returned `AllocatedPages` object will be empty.
    /// - If `at_page == self.end + 1`, the right returned `AllocatedPages` object will be empty.
    ///
    /// Returns an `Err` containing this `AllocatedPages` if `at_page` is otherwise out of bounds.
    pub fn split_at(self, at_page: Page) -> Result<(Self, Self), Self> {
        let (left, right) = if at_page == self.start {
            (PageRange::empty(), PageRange::new(at_page, self.start))
        } else if at_page == self.end + 1 {
            (PageRange::new(self.start, at_page - 1), PageRange::empty())
        } else if at_page > self.start && at_page <= self.end {
            (
                PageRange::new(self.start, at_page - 1),
                PageRange::new(at_page, self.end),
            )
        } else {
            return Err(self);
        };
        Ok((Self { pages: left }, Self { pages: right }))
    }
}

impl Deref for AllocatedPages {
    type Target = PageRange;
    fn deref(&self) -> &Self::Target {
        &self.pages
    }
}
