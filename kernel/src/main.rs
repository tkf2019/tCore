#![no_std]
#![no_main]
#![feature(naked_functions, asm_const)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod config;
mod cons;
mod error;
mod heap;
mod mm;
mod task;
mod trap;

extern crate alloc;

use config::BOOT_STACK_SIZE;

use crate::config::PHYSICAL_MEMORY_END;

/// Entry for kernel.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    // Initialize kernel stack in .bss section.
    #[link_section = ".bss.stack"]
    static mut STACK: [u8; BOOT_STACK_SIZE] = [0u8; BOOT_STACK_SIZE];

    core::arch::asm!(
        // Set stack pointer to the kernel stack.
        "la sp, {stack} + {stack_size}",
        // Jump to the main function.
        "j  {main}",
        stack_size = const BOOT_STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym rust_main,
        options(noreturn),
    )
}

/// Clear .bss
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

extern "C" fn rust_main() -> ! {
    // clear .bss
    clear_bss();
    cons::init();
    heap::init();
    mm::init();
    panic!("Panic")
}
