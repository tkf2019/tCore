#![no_std]
#![allow(unused)]

extern crate alloc;

mod register;
mod uipi;

pub use register::*;
pub use uipi::*;

pub unsafe fn uret() {
    core::arch::asm!("uret");
}