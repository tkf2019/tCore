use alloc::sync::Arc;
use kernel_sync::Mutex;
use log::info;
use spin::Lazy;

use crate::{
    arch::mm::PTEFlags,
    config::{MMIO, PHYSICAL_MEMORY_END},
    error::KernelResult,
    mm::pma::IdenticalPMA,
};

use super::MM;

pub static KERNEL_MM: Lazy<Mutex<MM>> = Lazy::new(|| Mutex::new(new_kernel().unwrap()));

/// Create new kernel address space
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
    mm.alloc_write_vma(
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
    mm.alloc_write_vma(
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
    mm.alloc_write_vma(
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
    mm.alloc_write_vma(
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
    mm.alloc_write_vma(
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

    // MMIO
    for (base, len) in MMIO {
        mm.alloc_write_vma(
            None,
            (*base).into(),
            (*base + *len).into(),
            PTEFlags::READABLE | PTEFlags::WRITABLE,
            Arc::new(Mutex::new(IdenticalPMA)),
        )?;
        info!("{:>10} [{:#x}, {:#x})", "mmio", base, base + len);
    }
    for (base, len) in crate::arch::MMIO {
        mm.alloc_write_vma(
            None,
            (*base).into(),
            (*base + *len).into(),
            PTEFlags::READABLE | PTEFlags::WRITABLE,
            Arc::new(Mutex::new(IdenticalPMA)),
        )?;
        info!("{:>10} [{:#x}, {:#x})", "arch mmio", base, base + len);
    }

    Ok(mm)
}
