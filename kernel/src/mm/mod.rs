use alloc::collections::LinkedList;
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use rust_lapper::Interval;
use spin::Mutex;
use tmm_rv::{AllocatedFrames, Frame, PTEFlags, Page, PageTable, PhysAddr, VirtAddr};
use vma::VMArea;

use crate::{config::TRAMPOLINE_VA, trap::trampoline};

use pmd::PMArea;
use vma::VMFlags;

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
    pub vma_list: LinkedList<VMA>,

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
    /// `Trampoline` is mapped to the same code section at first by default.
    /// `Trampoline` is not collected or recorded by VMAs, since this area cannot
    /// be unmapped or modified manually by user. We set the page table flags without
    /// [`PTEFlags::USER`] so that malicious user cannot jump to this area.
    pub fn new() -> Result<Self, &'static str> {
        let mm = Self {
            page_table: Arc::new(Mutex::new(PageTable::new()?)),
            vma_list: LinkedList::new(),
            vma_map: BTreeMap::new(),
            vma_cache: None,
            entry: VirtAddr::zero(),
            start_brk: VirtAddr::zero(),
            brk: VirtAddr::zero(),
        };
        mm.page_table.lock().map(
            VirtAddr::from(TRAMPOLINE_VA).into(),
            PhysAddr::from(trampoline as usize).into(),
            PTEFlags::READABLE | PTEFlags::EXECUTABLE,
        )?;
        Ok(mm)
    }

    pub fn alloc(&mut self, vma: VMA, data: &[u8]) -> Result<(), &'static str> {
        self.vma_list.push_back(vma);
        Ok(())
    }

    // Create user address space from ELF data.
    pub fn from_elf(elf_data: &[u8]) -> Result<Self, &'static str> {
        let mut mm = Self::new()?;
        let elf = xmas_elf::ElfFile::new(elf_data)?;
        let elf_header = elf.header;
        if elf_header.pt1.magic != [0x7f, 0x45, 0x4c, 0x46] {
            return Err("Invalid ELF!");
        }
        let ph_count = elf_header.pt2.ph_count();
        let mut last_page: Page = VirtAddr::zero().into();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let vm_flags: PTEFlags = PTEFlags::empty();
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    vm_flags |= PTEFlags::READABLE;
                }
                if ph_flags.is_write() {
                    vm_flags |= PTEFlags::WRITABLE;
                }
                if ph_flags.is_execute() {
                    vm_flags |= PTEFlags::EXECUTABLE;
                }
                // mm.vma_list.push_back(Arc::new(Mutex::new(VMArea::new(
                //     start_va,
                //     end_va,
                //     vm_flags,
                //     mm.page_table,
                // ))));

                last_page = end_va.into();
            }
        }
        // let max_end_va: VirtAddr = max_end_vpn.into();
        // let mut user_stack_base: usize = max_end_va.into();
        // user_stack_base += PAGE_SIZE;
        // (
        //     memory_set,
        //     user_stack_base,
        //     elf.header.pt2.entry_point() as usize,
        // )
    }
}
