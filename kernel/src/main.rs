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

use log::trace;

use crate::{
    arch::{__entry_others, start_hart},
    config::{BOOT_STACK_SIZE, CPU_NUM},
};

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
    trace!("Start executing tasks.");
    // Wake up other harts.
    for cpu_id in 0..CPU_NUM {
        if cpu_id != hartid {
            let entry = __entry_others as usize;
            trace!("Try to start hart {}", cpu_id);
            start_hart(cpu_id, entry, 0);
        }
    }
    // IDLE loop
    task::idle();
    unreachable!()
}

#[no_mangle]
pub extern "C" fn rust_main_others(hartid: usize) -> ! {
    trace!("(Secondary) Start executing tasks.");
    loop {}
}
