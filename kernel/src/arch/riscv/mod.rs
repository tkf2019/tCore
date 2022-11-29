mod stdio;

pub use stdio::{getchar, putchar, puts};

use crate::{
    config::{BOOT_STACK_SIZE, TOTAL_BOOT_STACK_SIZE},
    rust_main, rust_main_others,
};

// Initialize kernel stack in .bss section.
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

/// Get cpu id.
#[inline]
pub fn get_cpu_id() -> usize {
    let cpu_id;
    unsafe { core::arch::asm!("mv {0}, tp", out(reg) cpu_id) };
    cpu_id
}

/// Start other harts
#[inline]
pub fn start_hart(hartid: usize, entry: usize, opaque: usize) {
    let ret = sbi_rt::hart_start(hartid, entry, opaque);
    assert!(ret.is_ok(), "Failed to shart hart {}", hartid);
}
