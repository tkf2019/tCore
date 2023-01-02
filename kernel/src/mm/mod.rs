mod file;
mod flags;
mod kernel;
pub mod pma;
pub mod vma;

use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use core::{fmt, mem::size_of, slice};
use log::warn;
use spin::Mutex;
use tbuffer::UserBuffer;
use terrno::Errno;
use tmm_rv::{Frame, PTEFlags, Page, PageRange, PageTable, PhysAddr, VirtAddr};
use tsyscall::SyscallResult;

use crate::{config::*, error::*, mm::pma::LazyPMA, trap::__trampoline};

pub use file::BackendFile;
pub use flags::*;
pub use kernel::{init, KERNEL_MM};
use pma::PMArea;
use vma::VMArea;

pub struct MM {
    /// Holds the pointer to [`PageTable`].
    ///
    /// This object has the ownership of the page table. So the lifetime of [`PageTable`]
    /// depends on the [`MM`] tied to it. In `sys_vfork`, parent task will be blocked until
    /// the child task exits.
    ///
    /// Frames allocated in a page table will be dropped if the address space is
    /// destroyed to release the resources. See [`AllocatedFrames`].
    pub page_table: PageTable,

    /// List of [`VMArea`]s.
    vma_list: Vec<Option<VMArea>>,

    /// Recycled index of `vma_list`.
    vma_recycled: Vec<usize>,

    /// Find an unmapped [`VMArea`] with the target length quickly.
    vma_map: BTreeMap<VirtAddr, usize>,

    /// Last accessed [`VMArea`] cached for faster search with the prediction
    /// of memory locality.
    vma_cache: Option<usize>,

    /// Start virtual address of user code (known as entry point).
    pub entry: VirtAddr,

    /// Start virtual address of heap.
    pub start_brk: VirtAddr,

    /// Heap pointer managed by `sys_brk`.
    pub brk: VirtAddr,
}

/* Global operations */

impl MM {
    /// Create a new empty [`MM`] struct.
    ///
    /// `Trampoline` is mapped to the same code section at first by default.
    /// `Trampoline` is not collected or recorded by VMAs, since this area cannot
    /// be unmapped or modified manually by user. We set the page table flags without
    /// [`PTEFlags::USER`] so that malicious user cannot jump to this area.
    pub fn new() -> KernelResult<Self> {
        match PageTable::new() {
            Ok(page_table) => {
                let mut mm = Self {
                    page_table,
                    vma_list: Vec::new(),
                    vma_recycled: Vec::new(),
                    vma_map: BTreeMap::new(),
                    vma_cache: None,
                    entry: VirtAddr::zero(),
                    start_brk: VirtAddr::zero(),
                    brk: VirtAddr::zero(),
                };
                mm.page_table
                    .map(
                        VirtAddr::from(TRAMPOLINE_VA).into(),
                        PhysAddr::from(__trampoline as usize).into(),
                        PTEFlags::READABLE | PTEFlags::EXECUTABLE | PTEFlags::VALID,
                    )
                    .map_err(|err| {
                        warn!("{}", err);
                        KernelError::PageTableInvalid
                    })
                    .and(Ok(mm))
            }
            Err(_) => Err(KernelError::FrameAllocFailed),
        }
    }

    /// A warpper for `translate` in `PageTable`.
    pub fn translate(&mut self, va: VirtAddr) -> KernelResult<PhysAddr> {
        self.page_table
            .translate(va)
            .map_err(|_| KernelError::PageTableInvalid)
    }

    /// The number of virtual memory areas.
    pub fn map_count(&mut self) -> usize {
        self.vma_map.len()
    }

    pub fn mmap_min_addr(&self) -> VirtAddr {
        self.start_brk + USER_HEAP_SIZE
    }

    /// Writes to `[start_va, end_va)` using the page table of this address space.
    ///
    /// This function might be terminated if a page in this range is not mapped, thus
    /// the result is unpredictable. So it is marked as `unsafe` for further use.
    ///
    /// The length of `data` may be larger or smaller than the virtual memory range.
    unsafe fn write_vma(
        &mut self,
        data: &[u8],
        start_va: VirtAddr,
        end_va: VirtAddr,
    ) -> KernelResult {
        let end_ptr = data.len();
        let mut data_ptr: usize = 0;
        let mut curr_va = start_va;
        let mut curr_page = Page::from(start_va);
        let end_page = Page::from(end_va); // inclusive
        loop {
            let page_len: usize = if curr_page == end_page {
                (end_va - curr_va).into()
            } else {
                PAGE_SIZE - curr_va.page_offset()
            };

            // Copy data to allocated frames.
            let src = &data[data_ptr..end_ptr.min(data_ptr + page_len)];
            let dst = self.page_table.translate(curr_va).and_then(|pa| unsafe {
                Ok(slice::from_raw_parts_mut(
                    pa.value() as *mut u8,
                    page_len.min(end_ptr - data_ptr),
                ))
            });
            if dst.is_err() {
                return Err(KernelError::PageTableInvalid);
            }
            dst.unwrap().copy_from_slice(src);

            // Step to the next page.
            data_ptr += page_len;
            curr_va += page_len;
            curr_page += 1;

            if curr_va >= end_va || data_ptr >= end_ptr {
                break;
            }
        }
        Ok(())
    }

    /// Adds a new [`VMArea`] into the address space.
    ///
    /// This function does not create any memory map for the new area.
    pub fn add_vma(&mut self, vma: VMArea) -> KernelResult {
        if self.map_count() >= MAX_MAP_COUNT {
            return Err(KernelError::Errno(Errno::ENOMEM));
        }
        let mut index = self.vma_list.len();
        if !self.vma_recycled.is_empty() {
            index = self.vma_recycled.pop().unwrap();
            self.vma_map.insert(vma.start_va, index);
            self.vma_list[index] = Some(vma);
        } else {
            self.vma_map.insert(vma.start_va, index);
            self.vma_list.push(Some(vma));
        }
        self.vma_cache = Some(index);
        Ok(())
    }

    /// Allocates a new [`VMArea`] with the virtual range of `[start_va, end_va)`.
    ///
    /// Writes the data to the mapped physical areas without any check for overlaps.
    ///
    /// This function may be used when we try to initialize a kernel or user address
    /// space.
    pub fn alloc_write_vma(
        &mut self,
        data: Option<&[u8]>,
        start_va: VirtAddr,
        end_va: VirtAddr,
        flags: PTEFlags,
        pma: Arc<Mutex<dyn PMArea>>,
    ) -> KernelResult {
        let vma = VMArea::new(start_va, end_va, flags.into(), pma)?;
        vma.map_all(&mut self.page_table, flags)?;
        self.add_vma(vma)?;
        if let Some(data) = data {
            unsafe { self.write_vma(data, start_va, end_va)? };
        }
        Ok(())
    }

    /// Allocates a new [`VMArea`].
    ///
    /// # Argument
    /// - `start`: starting virtual address (aligned implicitly)
    /// - `end`: ending virtual address (aligned implicitly)
    /// - `flags`: page table entry flags
    /// - `anywhere`: if set, the given address range will be ignored
    /// - `backend`: if not none, a backend file will be managed by this area
    pub fn alloc_vma(
        &mut self,
        start: VirtAddr,
        end: VirtAddr,
        flags: VMFlags,
        anywhere: bool,
        backend: Option<BackendFile>,
    ) -> KernelResult<VirtAddr> {
        let len = end.value() - start.value();
        let (start, end) = if anywhere {
            let start = self.find_free_area(start, len)?;
            (start, start + len)
        } else {
            // Clear overlaps.
            self.do_munmap(start, len)?;
            (start, end)
        };

        let vma = VMArea::new(
            start,
            end,
            flags,
            Arc::new(Mutex::new(LazyPMA::new(page_index(start, end), backend)?)),
        )?;
        // There is no need to fllush TLB explicitly, because old maps
        // have been cleaned.
        self.add_vma(vma)?;

        Ok(start)
    }

    /// Finds a free area.
    pub fn find_free_area(&self, hint: VirtAddr, len: usize) -> KernelResult<VirtAddr> {
        let mut last_end = VirtAddr::zero();
        let min_addr = self.mmap_min_addr();
        for (_, index) in self.vma_map.range(hint..) {
            if let Some(vma) = &self.vma_list[*index] {
                if (vma.start_va - last_end).value() >= len && vma.start_va - len >= min_addr {
                    return Ok(vma.start_va - len);
                }
                last_end = vma.end_va;
            }
        }
        Err(KernelError::VMAAllocFailed)
    }

    /// Gets the virtual memory area that contains the virutal address.
    /// Applies the given operation to the target area.
    ///
    /// # Argument
    /// - `va`: virtual address that belongs to the area.
    /// - `op`: a mutable function that receives a mutable reference to the area.
    ///     - `0`: target virtual memory area
    ///     - `1`: page table in this address space
    ///     - `2`: index of the area
    ///
    /// # Error
    /// - [KernelError::PageUnmapped]: the page has not been mapped with `mmap`.
    pub fn get_vma<T>(
        &mut self,
        va: VirtAddr,
        mut op: impl FnMut(&mut VMArea, &mut PageTable, usize) -> KernelResult<T>,
    ) -> KernelResult<T> {
        // Find it in cache.
        if let Some(index) = self.vma_cache {
            if let Some(area) = &mut self.vma_list[index] {
                if area.contains(va) {
                    return op(area, &mut self.page_table, index);
                }
            }
        }

        // Find it in map.
        if let Some((_, index)) = self.vma_map.range(..=va).last() {
            if let Some(area) = &mut self.vma_list[*index] {
                if area.contains(va) {
                    // Update cache
                    self.vma_cache = Some(*index);
                    return op(area, &mut self.page_table, *index);
                }
            }
        }

        Err(KernelError::PageUnmapped)
    }

    /// Gets an ordered vector of the index of virtual memory areas that intersect
    /// with the range.
    pub fn get_vma_range(&mut self, start: VirtAddr, end: VirtAddr) -> KernelResult<Vec<usize>> {
        let mut v = Vec::new();

        // The first area that contains the start of range.
        if let Ok(start_area) = self.get_vma(start, |_, _, index| Ok(index)) {
            v.push(start_area);
        }

        // Find the areas whose starting virtual address is in the given range.
        // These areas must overlap with the given range.
        self.vma_map
            .range(start..end)
            .for_each(|(_, index)| v.push(*index));

        Ok(v)
    }

    /// Allocates a frame for mapped page.
    ///
    /// # Argument
    /// - `va`: starting virtual address.
    pub fn alloc_frame(&mut self, va: VirtAddr) -> KernelResult<Frame> {
        self.get_vma(va, |vma, pt, _| {
            vma.alloc_frame(Page::from(va), pt).map(|(frame, _)| frame)
        })
    }

    /// Allocates a range of frames for given virtual address range [start_va, end_va).
    ///
    /// # Argument
    /// - `start_va`: starting virtual address.
    /// - `end_va`: ending virtual address.
    pub fn alloc_frame_range(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
    ) -> KernelResult<Vec<Frame>> {
        let mut frames = Vec::new();
        for page in PageRange::from_virt_addr(start_va, (end_va - start_va).value()) {
            frames.push(
                self.get_vma(page.start_address(), |vma, pt, _| vma.alloc_frame(page, pt))
                    .map(|(frame, _)| frame)?,
            );
        }
        Ok(frames)
    }

    /// Allocates a type starting from the given virtual address.
    ///
    /// # Argument
    /// - `va`: starting virtual address where the data type locates.
    pub fn alloc_type<T: Sized>(&mut self, va: VirtAddr) -> KernelResult {
        self.alloc_frame_range(va, va + size_of::<T>())?;
        Ok(())
    }

    /// Allocates a type and writes data to the physical address.
    ///
    /// # Argument
    /// - `va`: starting virtual address where the data type locates.
    /// - `data`: reference of data type.
    pub fn alloc_write_type<T: Sized>(&mut self, va: VirtAddr, data: &T) -> KernelResult {
        let size = size_of::<T>();
        let end_va = va + size;
        self.alloc_frame_range(va, end_va)?;
        let data = unsafe { slice::from_raw_parts(data as *const T as *const _, size) };
        unsafe { self.write_vma(data, va, end_va)? };
        Ok(())
    }

    /// Gets bytes translated with the range of [start_va, start_va + len),
    /// which might cover several pages.
    ///
    /// The buffer may not be allocated with frames, so new frames will be
    /// allocated for further modifications on this buffer.
    ///
    /// # Argument
    /// - `va`: starting virtual address
    /// - `len`: total length of the buffer
    pub fn get_buf_mut(&mut self, va: VirtAddr, len: usize) -> KernelResult<UserBuffer> {
        let mut start_va = va;
        let end_va = start_va + len;
        let mut v = Vec::new();
        while start_va < end_va {
            let next_page = Page::from(start_va) + 1;
            let page_off = start_va.page_offset();
            let page_len: usize = (end_va - start_va)
                .min(next_page.start_address() - start_va)
                .into();
            let frame = self.alloc_frame(start_va)?;
            v.push(&mut frame.as_slice_mut()[page_off..page_off + page_len]);
            start_va += page_len;
        }
        Ok(UserBuffer::new(v))
    }

    /// Gets a string loaded from starting virtual address.
    ///
    /// # Argument
    /// - `va`: starting virtual address.
    /// - `len`: total length of the string.
    /// If the length is not provided, the string must end with a '\0'. New frames
    /// will be allocated until a '\0' occurs.
    pub fn get_str(&mut self, va: VirtAddr) -> KernelResult<String> {
        let mut string = String::new();
        let mut alloc = true;
        let mut frame = Frame::from(0);
        let mut va = va;
        loop {
            if va.page_offset() == 0 {
                alloc = true;
            }
            if alloc {
                frame = self.alloc_frame(va)?;
                alloc = false;
            }
            let ch: u8 = frame.as_slice_mut()[va.page_offset()];
            if ch == 0 {
                break;
            }
            string.push(ch as char);
            va += 1;
        }
        Ok(string)
    }
}

/* Syscall helpers */

/// Value aligned to the multiple of page size.
pub fn page_align(value: usize) -> usize {
    value & !(PAGE_SIZE - 1)
}

pub fn page_index(start_va: VirtAddr, va: VirtAddr) -> usize {
    Page::from(va).number() - Page::from(start_va).number()
}

impl MM {
    /// A helper for [`tsyscall::SyscallProc::brk`].
    pub fn do_brk(&mut self, brk: VirtAddr) -> SyscallResult {
        // Invalid brk
        if brk < self.start_brk {
            return Ok(self.brk.value());
        }

        // brk page aligned
        let new_page = Page::from(brk);
        let old_page = Page::from(self.brk);
        // No need to allocate new pages.
        if new_page == old_page {
            self.brk = brk;
            return Ok(brk.value());
        }

        // Always allow shrinking brk.
        if brk < self.brk {
            // Failed to unmap.
            if self
                .do_munmap(
                    (new_page + 1).start_address(),
                    (old_page.number() - new_page.number()) * PAGE_SIZE,
                )
                .is_err()
            {
                return Ok(self.brk.value());
            }
            self.brk = brk;
            return Ok(self.brk.value());
        }

        // Check against existing mmap mappings.
        if self.get_vma(brk - 1, |_, _, _| Ok(())).is_ok() {
            return Ok(self.brk.value());
        }

        // Initialize memory area
        if self.brk == self.start_brk {
            self.add_vma(VMArea::new(
                self.start_brk,
                self.start_brk + PAGE_SIZE,
                VMFlags::USER | VMFlags::READ | VMFlags::WRITE,
                Arc::new(Mutex::new(LazyPMA::new(1, None)?)),
            )?)?;
        }
        self.get_vma(self.start_brk, |vma, _, _| unsafe { vma.extend(brk) })
            .unwrap();
        self.brk = brk;
        Ok(brk.value())
    }

    /// A helper for [`tsyscall::SyscallProc::munmap`].
    pub fn do_munmap(&mut self, start: VirtAddr, len: usize) -> KernelResult {
        let len = page_align(len);
        if !start.is_aligned() || len == 0 {
            return Err(KernelError::Errno(Errno::EINVAL));
        }
        let end = start + len;

        // Find the target vma.
        let vma_range = self.get_vma_range(start, end)?;
        for index in vma_range {
            let mut need_remove = false;
            let vma = self.vma_list[index].as_mut().unwrap();
            let mut new_vma = None;
            // Limit exceeded.
            if start > vma.start_va && end < vma.end_va && self.vma_map.len() >= MAX_MAP_COUNT {
                return Err(KernelError::Errno(Errno::ENOMEM));
            }

            // Handle intersection cases.
            if vma.start_va >= start && vma.end_va <= end {
                vma.unmap_all(&mut self.page_table)?;
                need_remove = true;
            } else if vma.start_va < start && vma.end_va > end {
                let (mid, right) = vma.split(start, end)?;
                mid.unwrap().unmap_all(&mut self.page_table)?;
                new_vma = right;
            } else if vma.end_va > end {
                // vma starting address modified to end
                self.vma_map.remove(&vma.start_va);
                let (left, _) = vma.split(start, end)?;
                self.vma_map.insert(vma.start_va, index);
                left.unwrap().unmap_all(&mut self.page_table)?;
            } else {
                let (right, _) = vma.split(start, end)?;
                right.unwrap().unmap_all(&mut self.page_table)?;
            }

            // Remove the area from this address space.
            if need_remove {
                let vma = self.vma_list[index].take().unwrap();
                self.vma_recycled.push(index);
                self.vma_map.remove(&vma.start_va);
            }

            // Clear cache to avoid crashes.
            self.vma_cache = None;

            // A new area splitted from the original one.
            if let Some(new_vma) = new_vma {
                self.add_vma(new_vma)?;
            }
        }
        Ok(())
    }
}

/* Trap helpers */

impl MM {
    /// A page fault helper for [`crate::trap::user_trap_handler`].
    pub fn do_handle_page_fault(&mut self, va: VirtAddr, flags: VMFlags) -> KernelResult {
        self.get_vma(va, |vma, pt, _| {
            let (_, alloc) = vma.alloc_frame(Page::from(va), pt)?;
            // Page fault cannot be handled.
            if !alloc || !vma.flags.contains(flags) {
                return Err(KernelError::FatalPageFault);
            }
            Ok(())
        })
    }
}

/* Derives */

impl fmt::Debug for MM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\nAddress Space: entry=0x{:X?}, start_brk=0x{:X?}, brk=0x{:X?}",
            self.entry.value(),
            self.start_brk.value(),
            self.brk.value(),
        )?;
        for (_, index) in &self.vma_map {
            if let Some(vma) = &self.vma_list[*index] {
                writeln!(f, "{:#?}", vma)?;
            }
        }
        Ok(())
    }
}
