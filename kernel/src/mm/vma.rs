use alloc::sync::Arc;
use log::{info, warn};
use spin::Mutex;
use tmm_rv::{AllocatedPages, Frame, FrameRange, PTEFlags, Page, PageTable, VirtAddr};

use crate::error::{KernelError, KernelResult};

use super::pma::PMArea;

/// Represents an area in virtual address space with the range of [start_va, end_va).
pub struct VMArea {
    /// Access flags of this area.
    pub flags: PTEFlags,

    /// Start virtual address.
    pub start_va: VirtAddr,

    /// End virtual address.
    pub end_va: VirtAddr,

    /// This area has the ownership of [`AllocatedPages`].
    pages: AllocatedPages,

    /// Mapped to a physical memory area, with behaviors depending on the usage of this area.
    pma: Arc<Mutex<dyn PMArea>>,
}

impl VMArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        flags: PTEFlags,
        pma: Arc<Mutex<dyn PMArea>>,
    ) -> Self {
        Self {
            flags,
            start_va,
            end_va,
            pages: AllocatedPages::new(Page::from(start_va), Page::from(end_va - 1) + 1),
            pma,
        }
    }

    /// Maps the whole virtual memory area, throwing errors.
    pub fn map_this(&self, page_table: &mut PageTable) -> KernelResult {
        let pma = self.pma.lock();
        let frames = if pma.is_mapped() {
            self.pma.lock().get_frames()
        } else {
            FrameRange::new(
                Frame::from(self.pages.start.number()),
                Frame::from(self.pages.end.number()),
            )
            .range()
            .collect()
        };
        for (page, frame) in self.pages.range().zip(frames) {
            if page_table.map(page, frame, self.flags | PTEFlags::VALID).is_err() {
                warn!("Failed to create mapping: {:#x?} -> {:#x?}", page, frame);
                return Err(KernelError::PageTableInvalid);
            }
        }
        Ok(())
    }

    /// Unmaps the whole virtual memory area, escaping errors caused by page table walk.
    pub fn unmap_this(&self, page_table: &mut PageTable) -> KernelResult {
        for page in self.pages.range() {
            if page_table.unmap(page).is_err() {
                warn!("Map not exi");
            }
        }
        Ok(())
    }
}
