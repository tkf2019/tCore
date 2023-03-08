use kernel_sync::SpinLock;
use log::info;
use spin::Lazy;

use crate::{
    config::{MMIO, PHYSICAL_MEMORY_END},
    error::KernelResult,
    mm::VMFlags,
};

use super::MM;

pub static KERNEL_MM: Lazy<SpinLock<MM>> = Lazy::new(|| SpinLock::new(new_kernel().unwrap()));

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
        VMFlags::READ | VMFlags::EXEC | VMFlags::IDENTICAL,
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
        VMFlags::READ | VMFlags::IDENTICAL,
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
        VMFlags::READ | VMFlags::WRITE | VMFlags::IDENTICAL,
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
        VMFlags::READ | VMFlags::WRITE | VMFlags::IDENTICAL,
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
        VMFlags::READ | VMFlags::WRITE | VMFlags::IDENTICAL,
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
            VMFlags::READ | VMFlags::WRITE | VMFlags::IDENTICAL,
        )?;
        info!("{:>10} [{:#x}, {:#x})", "mmio", base, base + len);
    }
    for (base, len) in crate::arch::MMIO {
        mm.alloc_write_vma(
            None,
            (*base).into(),
            (*base + *len).into(),
            VMFlags::READ | VMFlags::WRITE | VMFlags::IDENTICAL,
        )?;
        info!("{:>10} [{:#x}, {:#x})", "arch mmio", base, base + len);
    }

    Ok(mm)
}
