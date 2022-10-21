use crate::{AllocatedFrames, Frame, Page, PageRange, PageTable};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

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
    page_table: Arc<PageTable>,

    /// Total continuous range of virtual pages.
    pages: PageRange,

    /// Virtual memory pages are mapped to allocated physical memory frames. This member holds
    /// the ownership of allocated frames and has the direct access to these frames. The lifetime
    /// of [`AllocatedFrames`] will be over as soon as the k-v pair is removed from this map and
    /// the physical memory resources will be deallocated automatically.
    map: BTreeMap<PageRange, AllocatedFrames>,
}


