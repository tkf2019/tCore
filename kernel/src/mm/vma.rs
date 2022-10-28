use core::error::Error;

use alloc::sync::{Weak, Arc};
use bitflags::bitflags;
use log::{warn, debug};
use spin::Mutex;
use tmm_rv::{AllocatedPages, VirtAddr, PageTable, PageRange, frame_alloc, AllocatedFrames, PTEFlags};

use crate::error::{KernelResult, KernelError, self};

/// Represents an area in virtual address space with the range of [start_va, end_va).
pub struct VMArea {
    /// The name for this area.
    name: &'static str,

    /// Access flags of this area.
    pub flags: PTEFlags,

    /// Start virtual address.
    pub start_va: VirtAddr,

    /// End virtual address.
    pub end_va: VirtAddr,

    /// This area has the ownership of [`AllocatedPages`].
    pages: AllocatedPages,

    /// The frames may not be allocated until  reads or writes

    /// Points to the previous [`VMArea`] in the data structure that maintains the order
    /// of these [`VMArea`]s in the same virtual adress space. In Linux, `mm_struct` uses
    /// a rb-tree to do interval search quickly. We can get the gap between `start_va` of
    /// this area and `end_va` of the previous one to improve the efficiency of searching
    /// unmapped areas.
    // prev: Weak<VMArea>,
}

impl VMArea {
    pub fn new(name: &'static str, start_va: VirtAddr, end_va: VirtAddr, flgas: PTEFlags) -> Self {
        Self {
            name,
            flags,
            start_va,
            end_va,
            pages: AllocatedPages::new(start_va.into(), (end_va - 1).into())
        }
    }

    /// Maps the whole virtual memory area, throwing errors.
    pub fn map_this(&self, page_table: Arc<Mutex<PageTable>>) -> KernelResult {
        match AllocatedFrames::new(self.pages.pages.size_in_pages()) {
            Ok(frames) => {
                let pt = page_table.lock();
                for (page, frame) in self.pages.pages.into_iter().zip(frames.into_iter()) {
                    if pt.map(page, frame, flags).is_err() {
                        warn!("Failed to create mapping: {:#x?} -> {:#x?}", page, frame);
                        return Err(KernelError::PageTableInvalid);
                    }
                }
                Ok(())
            },
            Err(err) => {
                warn!("{}", err);
                Err(KernelError::FrameAllocFailed)
            }
        }
    }

    /// Unmaps the whole virtual memory area, escaping errors caused by page table walk.
    pub fn unmap_this(&self, page_table: Arc<Mutex<PageTable>>) -> KernelResult {
        let pt = page_table.lock();
        for page in self.pages.pages.into_iter() {
            pt.unmap(page);
        }
        Ok(())
    }
}
