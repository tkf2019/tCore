/*!
- PAGE_SIZE is the minimum unit of user address space areas, so user
heap and stack have better be integral multiple of PAGE_SIZE.
*/
#![allow(unused)]

pub use tmm_rv::{LOW_MAX_VA, MAX_VA, PAGE_SIZE};

/* Global configurations */

/// Use guard page to avoid stack overflow.
pub const GUARD_PAGE: usize = PAGE_SIZE;

/// Trampoline takes up the highest page both in user and kernel space.
pub const TRAMPOLINE_VA: usize = MAX_VA - PAGE_SIZE + 1;

/* Kernel configurations */

/// Boot kernel size allocated in `_start` for single CPU.
pub const BOOT_STACK_SIZE: usize = 0x1_0000;

/// 512 KB Kernel stack size: 0x80 pages
pub const KERNEL_STACK_SIZE: usize = 0x0000;

/// 8 MB kernel heap size: 0x2_0000 pages
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

/// Used for kernel buddy system allocator
pub const KERNEL_HEAP_ORDER: usize = 32;

/// 256MB physical memory
pub const PHYSICAL_MEMORY_END: usize = 0x9000_0000;

/* User configurations */

/// 4 MB user heap size
pub const USER_HEAP_SIZE: usize = 0x40_0000;

/// 8 KB user stack size
pub const USER_STACK_SIZE: usize = 0x2000;

/// Task stacks starts at the next page of `Trampoline`
pub const USER_STACK_BASE: usize = TRAMPOLINE_VA;

/// End virtual address of `mmap` area
pub const USER_MMAP_END: usize = LOW_MAX_VA;
