use crate::{AllocatedFrames, Frame, PTEFlags, Page, PageRange, PageTable};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use spin::Mutex;

/// Represents a range of mapped virtual memory [`Page`]s.
///
/// These pages are allocated as the address space is created by kernel manager or else.
/// It must be initialized with a page table and share the ownership of the page table.
/// A single address space may have several [`MappedPages`] to represent different memory
/// sections such as `.bss`, `.data` and etc.
pub struct MappedPages {
    /// This object does not have the ownership of the page table. So the lifetime of
    /// [`PageTable`] depends on all mapped pages tied to it. Frames allocated in a page
    /// table will be dropped if the address space is destroyed to release the resources.
    page_table: Arc<Mutex<PageTable>>,

    /// Total continuous range of virtual pages.
    pages: PageRange,

    /// Virtual memory pages are mapped to allocated physical memory frames. Different 
    /// address spaces may share physical memory frames in our implementations. This member
    /// holds the pointer to allocated frames and has no direct access to these frames. 
    /// The lifetime of [`AllocatedFrames`] will be over as soon as the k-v pair is 
    /// removed from this map and the reference counter becomes zero. The physical memory
    /// resources will be deallocated automatically by `Drop` trait.
    map: BTreeMap<PageRange, Arc<AllocatedFrames>>,

    /// General permission for these pages.s
    flags: PTEFlags,
}

impl MappedPages {
    /// Create mapped pages from an existing page table
    pub fn new(page_table: Arc<Mutex<PageTable>>) -> Self {
        Self {
            page_table: page_table.clone(),
            pages: PageRange::empty(),
            map: BTreeMap::new(),
            flags: PTEFlags::empty(),
        }
    }
}
