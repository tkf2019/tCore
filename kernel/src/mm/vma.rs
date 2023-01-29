use core::fmt;
use log::info;
use tmm_rv::*;

use crate::{
    error::{KernelError, KernelResult},
    flush_tlb,
};

use super::{flags::*, page_index, pma::PMA};

/// Represents an area in virtual address space with the range of [start_va, end_va).
pub struct VMArea {
    /// Access flags of this area.
    pub flags: VMFlags,

    /// Start virtual address.
    pub start_va: VirtAddr,

    /// End virtual address.
    pub end_va: VirtAddr,

    /// This area has the ownership of [`AllocatedPageRange`].
    pages: AllocatedPageRange,

    /// Mapped to a physical memory area, with behaviors depending on the usage of this area.
    pma: PMA,
}

impl VMArea {
    /// Create new virtual memory area [start_va, end_va) with protection flags.
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        flags: VMFlags,
        pma: PMA,
    ) -> KernelResult<Self> {
        if end_va <= start_va || flags.is_empty() {
            return Err(KernelError::InvalidArgs);
        }
        Ok(Self {
            flags,
            start_va,
            end_va,
            pages: AllocatedPageRange::new(Page::from(start_va), Page::from(end_va - 1) + 1),
            pma,
        })
    }

    /// Updates members due to modification on address.
    pub fn adjust(&mut self) {
        self.pages =
            AllocatedPageRange::new(Page::from(self.start_va), Page::from(self.end_va - 1) + 1);
    }

    /// Returns if this area contains the virtual address.
    pub fn contains(&self, va: VirtAddr) -> bool {
        self.start_va <= va && self.end_va > va
    }

    /// Returns if this area covers the given virtual address range.
    pub fn covers(&self, start_va: VirtAddr, end_va: VirtAddr) -> bool {
        self.start_va <= start_va && self.end_va > end_va && start_va < end_va
    }

    /// Extends an area with new end.
    ///
    /// This function does not check if current area overlaps with an old area, thus  
    /// the result is unpredictable. So it is marked as `unsafe` for further use.
    pub unsafe fn extend(&mut self, new_end: VirtAddr) -> KernelResult {
        self.end_va = new_end;
        self.adjust();
        self.pma
            .lock()
            .extend(Page::from(new_end - 1).number() - self.pages.start.number() + 1)?;
        Ok(())
    }

    /// Maps the whole virtual memory area.
    ///
    /// Notice that this function will allocate frames directly to create map.
    ///
    /// This function flushes TLB entries each page, thus there is no need to
    /// call [`Self::flush_all`] explicitly.
    pub fn map_all(&self, pt: &mut PageTable, flags: PTEFlags) -> KernelResult {
        let mut pma = self.pma.lock();
        let frames = if pma.is_mapped() {
            pma.get_frames(true)?
        } else {
            FrameRange::new(
                Frame::from(self.pages.start.number()),
                Frame::from(self.pages.end.number()),
            )
            .range()
            .map(|frame| Some(frame))
            .collect()
        };
        for (page, frame) in self.pages.range().zip(frames) {
            if pt
                .map(
                    page,
                    frame.unwrap(),
                    PTEFlags::VALID | flags | self.flags.into(),
                )
                .is_err()
            {
                return Err(KernelError::PageTableInvalid);
            }
            flush_tlb(Some(page.start_address()));
        }
        Ok(())
    }

    /// Unmaps the whole virtual memory area, escaping errors.
    ///
    /// This function flushes TLB entries each page, thus there is no need to
    /// call [`Self::flush_all`] explicitly.
    pub fn unmap_all(&self, pt: &mut PageTable) -> KernelResult {
        self.pages.range().for_each(|page| {
            if pt.unmap(page).is_ok() {
                flush_tlb(Some(page.start_address()));
            }
        });
        Ok(())
    }

    /// Flushes all TLB entries.
    pub fn flush_all(&self) {
        self.pages
            .range()
            .for_each(|page| flush_tlb(Some(page.start_address())));
    }

    /// Allocates a frame for mapped page.
    ///
    /// Returns true if a new frame is really allocated.
    pub fn alloc_frame(&mut self, page: Page, pt: &mut PageTable) -> KernelResult<(Frame, bool)> {
        let (pte_pa, mut pte) = pt.create(page).map_err(|_| KernelError::PageTableInvalid)?;
        if !pte.flags().is_valid() {
            let mut pma = self.pma.lock();
            let index = page.number() - self.pages.start.number();
            let frame = pma.get_frame(index, true)?;
            pte.set_flags(
                PTEFlags::VALID | PTEFlags::ACCESSED | PTEFlags::DIRTY | self.flags.into(),
            );
            pte.set_ppn(&frame);
            pte.write(pte_pa);
            return Ok((frame, true));
        }
        Ok((pte.frame(), false))
    }

    /// Splits an area with aligned virtual address range.
    ///
    /// Six cases in total:
    /// 1. `start < end <= self.start < self.end` (do nothing)
    /// 2. `self.start < self.end <= start < end` (do nothing)
    /// 3. `start <= self.start < self.end <= end` (whole)
    /// 4. `self.start < start < end < self.end` (three pieces)
    /// 5. `self.start < start < self.end < end` (split right)
    /// 6. `start < self.start < end < self.end` (split left)
    ///
    /// # Argument
    /// - `start`: starting virtual address.
    /// - `end`: ending virtual address.
    ///
    /// # Return
    ///
    /// The first area is:
    /// - the middle part in case 4.
    /// - the right part in case 5.
    /// - the left part in case 6.
    ///
    /// The second area is the third part in case 4.
    pub fn split(
        &mut self,
        start: VirtAddr,
        end: VirtAddr,
    ) -> KernelResult<(Option<VMArea>, Option<VMArea>)> {
        if end <= self.start_va
            || self.end_va <= start
            || start <= self.start_va && self.end_va <= end
        {
            Ok((None, None))
        } else if self.start_va < start && end < self.end_va {
            let (mid_pma, right_pma) = self.pma.lock().split(
                Some(page_index(self.start_va, start)),
                Some(page_index(self.start_va, end)),
            )?;
            let mid_vma = mid_pma.and_then(|pma| Self::new(start, end, self.flags, pma).ok());
            let right_vma =
                right_pma.and_then(|pma| Self::new(end, self.end_va, self.flags, pma).ok());

            self.end_va = start;
            self.adjust();

            Ok((mid_vma, right_vma))
        } else if self.start_va < start && self.end_va < end {
            let (right_pma, _) = self
                .pma
                .lock()
                .split(Some(page_index(self.start_va, start)), None)?;
            let right_vma =
                right_pma.and_then(|pma| Self::new(start, self.end_va, self.flags, pma).ok());

            self.end_va = start;
            self.adjust();

            Ok((right_vma, None))
        } else if start < self.start_va && end < self.end_va {
            let (left_pma, _) = self
                .pma
                .lock()
                .split(None, Some(page_index(self.start_va, end)))?;
            let left_vma =
                left_pma.and_then(|pma| Self::new(self.start_va, end, self.flags, pma).ok());

            self.start_va = end;
            self.adjust();

            Ok((left_vma, None))
        } else {
            Err(KernelError::InvalidArgs)
        }
    }
}

/* Derives */

impl Clone for VMArea {
    fn clone(&self) -> Self {
        Self::new(self.start_va, self.end_va, self.flags, self.pma.clone()).unwrap()
    }
}

impl fmt::Debug for VMArea {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VMA [0x{:X?}, 0x{:X?}) => {:?}",
            self.start_va.value(),
            self.end_va.value(),
            self.flags
        )
    }
}
