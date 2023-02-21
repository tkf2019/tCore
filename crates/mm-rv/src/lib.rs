#![no_std]
#![feature(step_trait, core_intrinsics)]

extern crate alloc;

mod address;
mod config;
mod frame_alloc;
mod page_alloc;
mod page_table;

#[cfg(test)]
mod test;

pub use address::{Frame, FrameRange, Page, PageRange, PhysAddr, VirtAddr};
pub use config::*;
pub use frame_alloc::{
    frame_alloc, frame_dealloc, frame_init, AllocatedFrame, AllocatedFrameRange,
};
pub use page_alloc::AllocatedPageRange;
pub use page_table::{PTEFlags, PTWalkerFlags, PageTable, PageTableEntry};
