mod stdio;

pub use stdio::{getchar, putchar, puts};

use crate::{config::TOTAL_BOOT_STACK_SIZE, rust_main, rust_main_secondary};

// Initialize kernel stack in .bss section.
#[link_section = ".bss.stack"]
static mut STACK: [u8; TOTAL_BOOT_STACK_SIZE] = [0u8; TOTAL_BOOT_STACK_SIZE];

/// Entry for kernel.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn __entry(hartid: usize) -> ! {
    core::arch::asm!(
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
