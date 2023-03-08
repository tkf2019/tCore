pub mod flags;
mod init;

use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use vfs::{OpenFlags, Path};
use xmas_elf::{
    header,
    program::{self, SegmentData},
    ElfFile,
};

use crate::{
    arch::mm::{Page, VirtAddr, PAGE_SIZE},
    config::{ADDR_ALIGN, ELF_BASE_RELOCATE, USER_STACK_BASE, USER_STACK_SIZE},
    error::{KernelError, KernelResult},
    fs::open,
    mm::{VMFlags, MM},
    task::Task,
};

use self::{
    flags::AuxType,
    init::{InitInfo, InitStack},
};

/// Finds the user ELF in the given directory and creates the task.
pub fn from_args(dir: String, args: Vec<String>) -> KernelResult<Arc<Task>> {
    if args.len() < 1 {
        return Err(KernelError::InvalidArgs);
    }
    let name = args[0].as_str();
    let path = dir.clone() + "/" + name;
    let file = unsafe {
        open(Path::from(path), OpenFlags::O_RDONLY)
            .map_err(|errno| KernelError::Errno(errno))?
            .read_all()
    };
    Ok(Arc::new(Task::new(dir, file.as_slice(), args)?))
}

/// Create address space from elf.
pub fn from_elf(elf_data: &[u8], args: Vec<String>, mm: &mut MM) -> KernelResult<VirtAddr> {
    let elf = ElfFile::new(elf_data).unwrap();
    let elf_hdr = elf.header;

    // Check elf type
    if (elf_hdr.pt2.type_().as_type() != header::Type::Executable
        && elf_hdr.pt2.type_().as_type() != header::Type::SharedObject)
        // 64-bit format
        || elf_hdr.pt1.class() != header::Class::SixtyFour
        // 'E', 'L', 'F'
        || elf_hdr.pt1.magic != [0x7f, 0x45, 0x4c, 0x46]
        // RISC-V
        || elf_hdr.pt2.machine().as_machine() != header::Machine::RISC_V
    {
        return Err(KernelError::ELFInvalidHeader);
    }

    // Dynamic address
    let mut dyn_base = 0;
    let elf_base_va = if let Some(phdr) = elf
        .program_iter()
        .find(|phdr| phdr.get_type() == Ok(program::Type::Load) && phdr.offset() == 0)
    {
        let phdr_va = phdr.virtual_addr() as usize;
        if phdr_va != 0 {
            phdr_va
        } else {
            // If the first segment starts at 0, we need to put it at a higher address
            // to avoid conflicts with user programs.
            dyn_base = ELF_BASE_RELOCATE;
            ELF_BASE_RELOCATE
        }
    } else {
        0
    };

    // Load program header
    let mut max_page = Page::from(0);
    for phdr in elf.program_iter() {
        match phdr.get_type().unwrap() {
            program::Type::Load => {
                let start_va: VirtAddr = (phdr.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((phdr.virtual_addr() + phdr.mem_size()) as usize).into();
                max_page = Page::floor(end_va - 1) + 1;

                // Map flags
                let mut map_flags = VMFlags::USER;
                let phdr_flags = phdr.flags();
                if phdr_flags.is_read() {
                    map_flags |= VMFlags::READ;
                }
                if phdr_flags.is_write() {
                    map_flags |= VMFlags::WRITE;
                }
                if phdr_flags.is_execute() {
                    map_flags |= VMFlags::EXEC;
                }

                // Allocate a new virtual memory area
                let count = max_page - Page::floor(start_va).into();
                let data = match phdr.get_data(&elf).unwrap() {
                    SegmentData::Undefined(data) => data,
                    _ => return Err(KernelError::ELFInvalidSegment),
                };
                // Address may not be aligned.
                mm.alloc_write_vma(
                    Some(data),
                    start_va + dyn_base,
                    end_va + dyn_base,
                    map_flags,
                )?;
            }
            program::Type::Interp => {
                // let data = match phdr.get_data(&elf).unwrap() {
                //     SegmentData::Undefined(data) => data,
                //     _ => return Err(KernelError::ELFInvalidSegment),
                // };
                // let path = unsafe {raw_ptr_to}
            }
            _ => {}
        };
    }

    // .rela.dyn

    // .rela.plt

    // Set brk location
    mm.start_brk = max_page.start_address() + dyn_base;
    mm.brk = mm.start_brk;

    // Set user entry
    mm.entry = VirtAddr::from(elf_hdr.pt2.entry_point() as usize) + dyn_base;

    // Initialize user stack
    let ustack_base = USER_STACK_BASE - ADDR_ALIGN;
    let ustack_top = USER_STACK_BASE - USER_STACK_SIZE;
    mm.alloc_write_vma(
        None,
        ustack_top.into(),
        ustack_base.into(),
        VMFlags::READ | VMFlags::WRITE | VMFlags::USER,
    )?;
    let mut vsp = VirtAddr::from(ustack_base);
    let sp = mm.translate(vsp)?;
    let init_stack = InitStack::serialize(
        InitInfo {
            args,
            // TODO
            envs: Vec::new(),
            auxv: {
                let mut at_table = BTreeMap::new();
                at_table.insert(
                    AuxType::AT_PHDR,
                    elf_base_va + elf_hdr.pt2.ph_offset() as usize,
                );
                at_table.insert(AuxType::AT_PHENT, elf_hdr.pt2.ph_entry_size() as usize);
                at_table.insert(AuxType::AT_PHNUM, elf_hdr.pt2.ph_count() as usize);
                at_table.insert(AuxType::AT_RANDOM, 0);
                at_table.insert(AuxType::AT_PAGESZ, PAGE_SIZE);
                at_table
            },
        },
        sp,
        vsp,
    );
    vsp -= init_stack.len();
    Ok(vsp)
}
