/*
 * @Date: 2021-12-04 20:53:00
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-04 20:56:44
 * @FilePath: /OS/tCore/src/main.rs
 */
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

    r0::zero_bss(&mut sbss, &mut ebss);

    println!("Welcome to Rust Boot Loader");

    let entry: usize = 0x80200000;
    println!("Running on S7 Core: hartid={}", hartid);
}
