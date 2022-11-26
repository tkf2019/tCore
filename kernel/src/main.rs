#![no_std]
#![no_main]
#![feature(naked_functions, asm_const)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]

#[path = "arch/riscv/mod.rs"]
mod arch;
mod config;
mod cons;
mod driver;
mod error;
mod fs;
mod heap;
mod mm;
mod syscall;
mod task;
mod trap;

extern crate alloc;

use config::BOOT_STACK_SIZE;
use log::trace;

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

#[no_mangle]
pub extern "C" fn rust_main(hartid: usize) -> ! {
    // Clear .bss
    clear_bss();
    // Initialization
    cons::init();
    heap::init();
    mm::init();
    task::init();
    trace!("[CPU {}] Start executing tasks.", hartid);
    // IDLE loop
    task::idle();
    unreachable!()
}

#[no_mangle]
pub extern "C" fn rust_main_secondary() -> ! {
    unimplemented!()
}
