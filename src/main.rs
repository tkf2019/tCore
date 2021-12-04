//! RBL for S74

#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]

#[macro_use]
mod serial;
// mod clint;
// mod trap;

use core::panic::PanicInfo;

#[no_mangle]
pub unsafe extern "C" fn boot_hart_zero(hartid: usize, dtb: usize) -> ! {
    extern "C" {
        static mut sbss: u32;
        static mut ebss: u32;
    }
}
