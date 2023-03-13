use core::fmt;

use alloc::{sync::Arc, vec::Vec};
use log::warn;

use crate::{
    arch::{flush_tlb, mm::*},
    config::USER_MAX_PAGES,
    error::{KernelError, KernelResult},
};

use super::{flags::*, page_count, page_index, page_range, MmapFile};

/// Represents an area in virtual address space with the range of [start_va, end_va).
pub struct VMArea {
    /// Access flags of this area.
    pub flags: VMFlags,

    /// Start virtual address.
    pub start_va: VirtAddr,

    /// End virtual address.
    pub end_va: VirtAddr,

    /// Mapped to a allocated frames.
    pub frames: Vec<Option<Arc<AllocatedFrame>>>,

    /// Backed by file wihch can be None.
    pub file: Option<Arc<MmapFile>>,
}

impl VMArea {
    /// Creates a new virtual memory area [start_va, end_va) with protection flags.
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        flags: VMFlags,
        frames: Vec<Option<Arc<AllocatedFrame>>>,
        file: Option<Arc<MmapFile>>,
    ) -> KernelResult<Self> {
        if end_va <= start_va || flags.is_empty() {
            return Err(KernelError::InvalidArgs);
        }
        Ok(Self {
            flags,
            start_va,
            end_va,
            frames,
            file,
        })
    }

    /// Creates a new [`VMArea`] with frames allocated lazily.
    pub fn new_lazy(
        start_va: VirtAddr,
        end_va: VirtAddr,
        flags: VMFlags,
        file: Option<Arc<MmapFile>>,
    ) -> KernelResult<Self> {
        let count = page_count(start_va, end_va);
        if end_va <= start_va || flags.is_empty() || count == 0 || count > USER_MAX_PAGES {
            return Err(KernelError::InvalidArgs);
        }

        let mut frames = Vec::new();
        frames.resize_with(count, || None);

        Ok(Self {
            flags,
            start_va,
            end_va,
            frames,
            file,
        })
    }

    /// Creates a new [`VMArea`] with frames allocated in advance.
    pub fn new_fixed(start_va: VirtAddr, end_va: VirtAddr, flags: VMFlags) -> KernelResult<Self> {
        let count = page_count(start_va, end_va);
        if end_va <= start_va || flags.is_empty() || count == 0 || count > USER_MAX_PAGES {
            return Err(KernelError::InvalidArgs);
        }

        let mut frames = Vec::new();

        if !flags.contains(VMFlags::IDENTICAL) {
            frames.resize_with(count, || Some(Arc::new(AllocatedFrame::new(true).unwrap())));
        }

        Ok(Self {
            flags,
            start_va,
            end_va,
            frames,
            file: None,
        })
    }

    /// Returns the size of this [`VMArea`] in pages.
    pub fn size_in_pages(&self) -> usize {
        page_count(self.start_va, self.end_va)
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
    pub unsafe fn extend(&mut self, new_end: VirtAddr) {
        self.end_va = new_end;
        self.frames.resize_with(self.size_in_pages(), || None);
    }

    /// Gets the frame by index.
    pub fn get_frame(&mut self, index: usize, alloc: bool) -> KernelResult<Frame> {
        if let Some(frame) = &self.frames[index] {
            Ok((*frame.as_ref()).clone())
        } else if alloc {
            let frame = AllocatedFrame::new(true).map_err(|_| KernelError::FrameAllocFailed)?;
            if let Some(file) = &self.file {
                if file.read(index * PAGE_SIZE, frame.as_slice_mut()).is_none() {
                    return Err(KernelError::VMAFailedIO);
                }
            }
            let frame_inner = frame.clone();
            // ownership moved
            self.frames[index] = Some(Arc::new(frame));
            Ok(frame_inner)
        } else {
            Err(KernelError::FrameNotFound)
        }
    }

    /// Reclaims the frame by index, writing back to file if before the [`AllocatedFrame`] dropped.
    pub fn reclaim_frame(&mut self, index: usize) {
        if let Some(frame) = self.frames[index].take() {
            if self.file.is_some() && Arc::strong_count(&frame) == 1 {
                // TODO: wirte if dirty
                self.file
                    .as_ref()
                    .unwrap()
                    .write(index * PAGE_SIZE, frame.as_slice());
            }
        }
    }

    /// Gets all frames of this [`VMArea`].
    pub fn get_frames(&mut self, alloc: bool) -> KernelResult<Vec<Option<Frame>>> {
        if self.flags.contains(VMFlags::IDENTICAL) {
            let start = Frame::from(Page::from(self.start_va).number());
            Ok(FrameRange::new(start, start + self.size_in_pages())
            .range()
            .map(|frame| Some(frame))
            .collect())
        } else {
            let mut v = Vec::new();
            for frame in &mut self.frames {
                if let Some(frame) = frame {
                    v.push(Some((*frame.as_ref()).clone()))
                } else {
                    if alloc {
                        let new_frame = frame.insert(Arc::new(
                            AllocatedFrame::new(true).map_err(|_| KernelError::FrameAllocFailed)?,
                        ));
                        v.push(Some((*new_frame.as_ref()).clone()))
                    } else {
                        v.push(None);
                    }
                }
            }
            Ok(v)
        }
    }

    /// Maps the whole virtual memory area.
    ///
    /// Notice that this function will allocate frames directly to create map.
    ///
    /// This function flushes TLB entries each page, thus there is no need to
    /// call [`Self::flush_all`] explicitly.
    pub fn map_all(&mut self, pt: &mut PageTable, flags: PTEFlags, alloc: bool) -> KernelResult {
        for (page, frame) in page_range(self.start_va, self.end_va)
            .range()
            .zip(self.get_frames(alloc)?)
        {
            if frame.is_some() {
                pt.map(page, frame.unwrap(), PTEFlags::VALID | flags)
                    .map_err(|err| {
                        warn!("{}", err);
                        KernelError::PageTableInvalid
                    })?;
            }
        }
        flush_tlb(None);
        Ok(())
    }

    /// Unmaps the whole virtual memory area, escaping errors.
    ///
    /// This function flushes TLB entries each page, thus there is no need to
    /// call [`Self::flush_all`] explicitly.
    pub fn unmap_all(&self, pt: &mut PageTable) -> KernelResult {
        page_range(self.start_va, self.end_va)
            .range()
            .for_each(|page| pt.unmap(page));
        flush_tlb(None);
        Ok(())
    }

    /// Allocates a frame for mapped page.
    ///
    /// Returns true if a new frame is really allocated.
    pub fn alloc_frame(
        &mut self,
        page: Page,
        pt: &mut PageTable,
        overwrite: bool,
    ) -> KernelResult<(Frame, bool)> {
        let (pte_pa, mut pte) = pt.create(page).map_err(|_| KernelError::PageTableInvalid)?;
        if !pte.flags().is_valid() || overwrite {
            let index = page.number() - Page::from(self.start_va).number();

            // reclaims the old frame
            let frame = if pte.flags().is_valid() {
                let old = self.get_frame(index, false)?;
                self.reclaim_frame(index);
                let new = self.get_frame(index, true)?;

                // copy on write
                new.as_slice_mut().copy_from_slice(old.as_slice());

                // no cow from now on
                self.flags.remove(VMFlags::CLONED);

                new
            } else {
                self.get_frame(index, true)?
            };

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
        let start_idx = page_index(self.start_va, start);
        let end_idx = page_index(self.start_va, end);

        if end <= self.start_va
            || self.end_va <= start
            || start <= self.start_va && self.end_va <= end
        {
            Ok((None, None))
        } else if self.start_va < start && end < self.end_va {
            let right_vma = Some(
                Self::new(
                    end,
                    self.end_va,
                    self.flags,
                    self.frames.drain(end_idx..).collect(),
                    self.file
                        .as_ref()
                        .map(|file| Arc::new(file.split(end_idx * PAGE_SIZE))),
                )
                .unwrap(),
            );
            let mid_vma = Some(
                Self::new(
                    start,
                    end,
                    self.flags,
                    self.frames.drain(start_idx..).collect(),
                    self.file
                        .as_ref()
                        .map(|file| Arc::new(file.split(start_idx * PAGE_SIZE))),
                )
                .unwrap(),
            );

            self.end_va = start;

            Ok((mid_vma, right_vma))
        } else if self.start_va < start && self.end_va < end {
            let right_vma = Some(
                Self::new(
                    start,
                    self.end_va,
                    self.flags,
                    self.frames.drain(start_idx..).collect(),
                    self.file
                        .as_ref()
                        .map(|file| Arc::new(file.split(start_idx * PAGE_SIZE))),
                )
                .unwrap(),
            );

            self.end_va = start;

            Ok((right_vma, None))
        } else if start < self.start_va && end < self.end_va {
            let left_vma = Some(
                Self::new(
                    self.start_va,
                    end,
                    self.flags,
                    self.frames.drain(..end_idx).collect(),
                    self.file.as_ref().map(|file| Arc::new(file.split(0))),
                )
                .unwrap(),
            );

            self.start_va = end;
            self.file = self
                .file
                .as_ref()
                .map(|file| Arc::new(file.split(end_idx * PAGE_SIZE)));

            Ok((left_vma, None))
        } else {
            Err(KernelError::InvalidArgs)
        }
    }
}

/* Derives */

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
