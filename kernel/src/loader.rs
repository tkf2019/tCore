use alloc::{string::String, sync::Arc, vec::Vec};
use spin::Mutex;
use tmm_rv::{PTEFlags, Page, VirtAddr};
use tvfs::OpenFlags;
use xmas_elf::{
    header,
    program::{self, SegmentData},
    ElfFile,
};

use crate::{
    error::{KernelError, KernelResult},
    fs::open,
    mm::{pma::FixedPMA, MM},
    task::{kstack_alloc, pid_alloc, Task},
};

/// Finds the user ELF in the given directory and creates the task.
pub fn from_args(dir: String, args: Vec<String>) -> KernelResult<Arc<Task>> {
    if args.len() < 1 {
        return Err(KernelError::InvalidArgs);
    }
    let name = args[0].as_str();
    let path = dir + "/" + name;
    let file = unsafe {
        open(path.as_str(), OpenFlags::O_RDONLY)
            .map_err(|errno| KernelError::ErrNO(errno))?
            .read_all()
    };
    let elf_data = unsafe { file.as_slice() };
    // New process identification
    let pid = pid_alloc();
    // New kernel stack for user task
    let kstack = kstack_alloc();
    Ok(Arc::new(Task::new(pid, kstack, elf_data, args)?))
}

/// Create address space from elf.
pub fn from_elf(elf_data: &[u8], args: Vec<String>) -> KernelResult<MM> {
    let mut mm = MM::new()?;
    let elf = ElfFile::new(elf_data).unwrap();
    let elf_header = elf.header;

    // Check elf type
    if (elf_header.pt2.type_().as_type() != header::Type::Executable
        && elf_header.pt2.type_().as_type() != header::Type::SharedObject)
        // 64-bit format
        || elf_header.pt1.class() != header::Class::SixtyFour
        // 'E', 'L', 'F'
        || elf_header.pt1.magic != [0x7f, 0x45, 0x4c, 0x46]
        // RISC-V
        || elf_header.pt2.machine().as_machine() != header::Machine::RISC_V
    {
        return Err(KernelError::ELFInvalid);
    }

    // Load program header
    let mut max_page = Page::from(0);
    for ph in elf.program_iter() {
        match ph.get_type().unwrap() {
            program::Type::Load => {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                max_page = Page::floor(end_va - 1) + 1;

                // Map flags
                let mut map_flags: PTEFlags = PTEFlags::USER_ACCESSIBLE;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_flags |= PTEFlags::READABLE;
                }
                if ph_flags.is_write() {
                    map_flags |= PTEFlags::WRITABLE;
                }
                if ph_flags.is_execute() {
                    map_flags |= PTEFlags::EXECUTABLE;
                }

                // Allocate a new virtual memory area
                let count = max_page - Page::floor(start_va).into();
                let data = match ph.get_data(&elf).unwrap() {
                    SegmentData::Undefined(data) => data,
                    _ => return Err(KernelError::ELFInvalid),
                };
                mm.alloc_write(
                    Some(data),
                    start_va,
                    end_va,
                    map_flags,
                    Arc::new(Mutex::new(FixedPMA::new(count.number())?)),
                )?;
            }
            program::Type::Interp => {}
            _ => {}
        };
    }
    // Set brk location
    mm.start_brk = max_page.into();
    mm.brk = mm.start_brk;
    mm.entry = (elf_header.pt2.entry_point() as usize).into();
    Ok(mm)
}
