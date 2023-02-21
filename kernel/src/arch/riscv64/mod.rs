pub mod mm {
    pub use mm_rv::*;
}
pub mod timer;
pub mod trap;
pub mod uintr;

use mm_rv::*;
use riscv::asm::{sfence_vma, sfence_vma_all};

use crate::{
    config::{BOOT_STACK_SIZE, PHYSICAL_MEMORY_END, TOTAL_BOOT_STACK_SIZE},
    mm::KERNEL_MM,
    rust_main, rust_main_others,
};

use self::uintr::{test_uintr, UINTC_BASE, UINTC_SIZE};

/// Initialize kernel stack in .bss section.
#[link_section = ".bss.stack"]
static mut STACK: [u8; TOTAL_BOOT_STACK_SIZE] = [0u8; TOTAL_BOOT_STACK_SIZE];

/// Entry for the first kernel.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn __entry(hartid: usize) -> ! {
    core::arch::asm!(
        // Use tp to save hartid
        "mv tp, a0",
        // Set stack pointer to the kernel stack.
        "la sp, {stack} + {stack_size}",
        // Jump to the main function.
        "j  {main}",
        stack_size = const TOTAL_BOOT_STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym rust_main,
        options(noreturn),
    )
}

/// Entry for other kernels.
#[naked]
#[no_mangle]
pub unsafe extern "C" fn __entry_others(hartid: usize) -> ! {
    core::arch::asm!(
        // Use tp to save hartid
        "mv tp, a0",
        // Set stack pointer to the kernel stack.
        "
        la a1, {stack}
        li t0, {total_stack_size}
        li t1, {stack_size}
        mul sp, a0, t1
        sub sp, t0, sp
        add sp, a1, sp
        ",
        // Jump to the main function.
        "j  {main}",
        total_stack_size = const TOTAL_BOOT_STACK_SIZE,
        stack_size       = const BOOT_STACK_SIZE,
        stack            =   sym STACK,
        main             =   sym rust_main_others,
        options(noreturn),
    )
}

/// Flushes tlb
pub fn flush_tlb(va: Option<VirtAddr>) {
    if let Some(va) = va {
        unsafe { sfence_vma(0, va.value()) };
    } else {
        unsafe { sfence_vma_all() };
    }
}

/// Gets cpu id.
#[inline]
pub fn get_cpu_id() -> usize {
    let cpu_id: usize;
    unsafe { core::arch::asm!("mv {0}, tp", out(reg) cpu_id) };
    cpu_id
}

/// Starts other harts.
#[inline]
pub fn start_hart(hartid: usize, entry: usize, opaque: usize) {
    let ret = sbi_rt::hart_start(hartid, entry, opaque);
    assert!(ret.is_ok(), "Failed to shart hart {}", hartid);
}

/// Architecture based MMIO.
pub const MMIO: &[(usize, usize)] = &[
    (UINTC_BASE, UINTC_SIZE), // User interrupt controller
];

/// Architecture based tests and initialization.
pub fn init(hartid: usize, is_main: bool) {
    assert_eq!(get_cpu_id(), hartid);
    
    // Initialize global frame allocator once.
    if is_main {
        extern "C" {
            fn ekernel();
        }
        frame_init(
            Frame::ceil(PhysAddr::from(ekernel as usize)).into(),
            Frame::floor(PhysAddr::from(PHYSICAL_MEMORY_END)).into(),
        );
    }

    // Set kernel trap entry.
    trap::set_kernel_trap();

    // Activate virtual address translation and protectiong using kernel page table.
    let satp = KERNEL_MM.lock().page_table.satp();
    riscv::register::satp::write(satp);
    flush_tlb(None);

    // Test user interrupt supports.
    #[cfg(feature = "uintr")]
    unsafe { test_uintr(hartid) };
}
