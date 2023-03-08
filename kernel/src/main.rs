#![no_std]
#![no_main]
#![feature(naked_functions, asm_const)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]
#![feature(sync_unsafe_cell)]
#![allow(dead_code)]

mod config;
mod cons;
mod driver;
mod error;
mod fs;
mod heap;
mod loader;
mod mm;
mod syscall;
mod task;
mod tests;

#[path = "arch/riscv64/mod.rs"]
#[cfg(target_arch = "riscv64")]
mod arch;

extern crate alloc;

use log::info;

use crate::config::{CPU_NUM, IS_TEST_ENV};

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
    clear_bss();
    cons::init();
    // Initialize global heap allocator.
    heap::init();
    // Other initializations
    arch::init(hartid, true);
    // Initialize oscomp testcases, which will be loaded from disk.
    if IS_TEST_ENV {
        oscomp::init(oscomp::testcases::LIBC_STATIC_TESTCASES);
    }
    // Initialize the first task.
    task::init();
    // Wake up other harts.
    for cpu_id in 0..CPU_NUM {
        if cpu_id != hartid {
            let entry = arch::__entry_others as usize;
            info!("Try to start hart {}", cpu_id);
            arch::start_hart(cpu_id, entry, 0);
        }
    }
    // IDLE loop
    unsafe { task::idle() };
}

#[no_mangle]
pub extern "C" fn rust_main_others(hartid: usize) -> ! {
    // Other initializations.

    arch::init(hartid, false);
    info!("(Secondary) Start executing tasks.");
    // IDLE loop
    unsafe { task::idle() };
}
