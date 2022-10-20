#![no_std]
#![feature(step_trait, core_intrinsics)]
#![allow(unused)]

extern crate alloc;

mod address;
mod config;
mod frame;
mod memory;
mod page_table;

#[cfg(test)]
mod test;

pub use address::{Frame, FrameRange, Page, PageRange, PhysAddr, VirtAddr};
pub use config::*;
pub use page_table::*;
