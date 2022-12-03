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
mod loader;
mod mm;
mod syscall;
mod task;
mod trap;

extern crate alloc;

use log::trace;
use tmm_rv::{frame_init, Frame, PhysAddr};

use crate::{
    arch::{__entry_others, start_hart},
    config::{CPU_NUM, IS_TEST_ENV, PHYSICAL_MEMORY_END},
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
    clear_bss();
    cons::init();
    // Initialize global heap allocator.
    heap::init();
    // Initialize global frame allocator.
    extern "C" {
        fn ekernel();
    }
    frame_init(
        Frame::ceil(PhysAddr::from(ekernel as usize)).into(),
        Frame::floor(PhysAddr::from(PHYSICAL_MEMORY_END)).into(),
    );
    // Activate kernel virtual address space.
    mm::init();
    // Set kernel trap entry.
    trap::set_kernel_trap();
    // Initialize oscomp testcases, which will be loaded from disk.
    if IS_TEST_ENV {
        oscomp::init(oscomp::testcases::LIBC_STATIC_TESTCASES);
    }
    // Initialize the first task.
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
}

#[no_mangle]
pub extern "C" fn rust_main_others(hartid: usize) -> ! {
    // Activate kernel virtual address space.
    mm::init();
    // Set kernel trap entry.
    trap::set_kernel_trap();
    trace!("(Secondary) Start executing tasks.");
    // IDLE loop
    task::idle();
}
