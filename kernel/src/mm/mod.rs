use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::slice;
use log::{debug, info, warn};
use spin::{Lazy, Mutex};
use tmm_rv::{frame_init, Frame, PTEFlags, Page, PageTable, PhysAddr, VirtAddr};
use vma::VMArea;

use crate::{
    config::{PAGE_SIZE, PHYSICAL_MEMORY_END, TRAMPOLINE_VA},
    error::{KernelError, KernelResult},
    println,
    trap::trampoline,
};

use self::pma::{IdenticalPMA, PMArea};

mod pma;
mod vma;

type VMA = Arc<Mutex<VMArea>>;

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
    pub vma_list: Vec<VMA>,

    /// Find an unmapped [`VMArea`] with the target length quickly
    pub vma_map: BTreeMap<VirtAddr, VMA>,

    /// Last accessed [`VMArea`] cached for faster search with the prediction
    /// of memory locality.
    pub vma_cache: Option<VMA>,

    /// Start virtual address of user code (known as entry point).
    pub entry: VirtAddr,

    /// Start virtual address of heap.
    pub start_brk: VirtAddr,

    /// Heap pointer managed by user `sys_brk` and `sys_sbrk`.
    pub brk: VirtAddr,
}

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
                    vma_map: BTreeMap::new(),
                    vma_cache: None,
                    entry: VirtAddr::zero(),
                    start_brk: VirtAddr::zero(),
                    brk: VirtAddr::zero(),
                };
                mm.page_table
                    .map(
                        VirtAddr::from(TRAMPOLINE_VA).into(),
                        PhysAddr::from(trampoline as usize).into(),
                        PTEFlags::READABLE | PTEFlags::EXECUTABLE,
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

    /// Write to `[start_va, end_va)` using the page table of this address space.
    /// The length of `data` may be larger or smaller than the virtual memory range.
    ///
    /// Returns [`KernelError::PageTableInvalid`] if it attempts to writing to an unmapped area.
    pub fn write(&mut self, data: &[u8], start_va: VirtAddr, end_va: VirtAddr) -> KernelResult {
        debug!("Write to virtual range: {:?} -> {:?}", start_va, end_va);

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
                Ok(slice::from_raw_parts_mut(pa.value() as *mut u8, page_len))
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

    /// Allocate a new [`VMArea`] and write the data to the mapped physical areas.
    pub fn alloc_write(
        &mut self,
        data: Option<&[u8]>,
        start_va: VirtAddr,
        end_va: VirtAddr,
        flags: PTEFlags,
        pma: Arc<Mutex<dyn PMArea>>,
    ) -> KernelResult {
        let vma = Arc::new(Mutex::new(VMArea::new(start_va, end_va, flags, pma)));
        vma.lock().map_this(&mut self.page_table)?;
        // Create new references
        self.vma_list.push(vma.clone());
        self.vma_map.insert(start_va, vma.clone());
        self.vma_cache = Some(vma.clone());
        if let Some(data) = data {
            self.write(data, start_va, end_va)?;
        }
        Ok(())
    }
}

pub static KERNEL_MM: Lazy<MM> = Lazy::new(||new_kernel().unwrap());

fn new_kernel() -> KernelResult<MM> {
    // Physical memory layout.
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss_with_stack();
        fn ebss();
        fn ekernel();
    }

    let mut mm = MM::new()?;

    // Map kernel .text section
    mm.alloc_write(
        None,
        (stext as usize).into(),
        (etext as usize).into(),
        PTEFlags::READABLE | PTEFlags::EXECUTABLE,
        Arc::new(Mutex::new(IdenticalPMA)),
    )?;
    info!(
        "{:>10} [{:#x}, {:#x})",
        ".text", stext as usize, etext as usize
    );

    // Map kernel .rodata section
    mm.alloc_write(
        None,
        (srodata as usize).into(),
        (erodata as usize).into(),
        PTEFlags::READABLE,
        Arc::new(Mutex::new(IdenticalPMA)),
    )?;
    info!(
        "{:>10} [{:#x}, {:#x})",
        ".rodata", srodata as usize, erodata as usize
    );

    // Map kernel .data section
    mm.alloc_write(
        None,
        (sdata as usize).into(),
        (edata as usize).into(),
        PTEFlags::READABLE | PTEFlags::WRITABLE,
        Arc::new(Mutex::new(IdenticalPMA)),
    )?;
    info!(
        "{:>10} [{:#x}, {:#x})",
        ".data", sdata as usize, edata as usize
    );

    // Map kernel .bss section
    mm.alloc_write(
        None,
        (sbss_with_stack as usize).into(),
        (ebss as usize).into(),
        PTEFlags::READABLE | PTEFlags::WRITABLE,
        Arc::new(Mutex::new(IdenticalPMA)),
    )?;
    info!(
        "{:>10} [{:#x}, {:#x})",
        ".bss", sbss_with_stack as usize, ebss as usize
    );

    // Physical memory area
    mm.alloc_write(
        None,
        (ekernel as usize).into(),
        PHYSICAL_MEMORY_END.into(),
        PTEFlags::READABLE | PTEFlags::WRITABLE,
        Arc::new(Mutex::new(IdenticalPMA)),
    )?;
    info!(
        "{:>10} [{:#x}, {:#x})",
        "mem", ekernel as usize, PHYSICAL_MEMORY_END
    );

    Ok(mm)
}

/// Initialize global frame allocator.
/// Activate virtual address translation and protectiong using kernel page table.
pub fn init() {
    info!("Initializing kernel address space...");

    extern "C" {
        fn ekernel();
    }
    frame_init(
        Frame::ceil(PhysAddr::from(ekernel as usize)).into(),
        Frame::floor(PhysAddr::from(PHYSICAL_MEMORY_END)).into(),
    );

    let satp = KERNEL_MM.page_table.satp();
    unsafe {
        riscv::register::satp::write(satp);
        core::arch::asm!("sfence.vma");
    }

    info!("Kernel address space initialized successfully.");
}
