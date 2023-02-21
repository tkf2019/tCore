use crate::arch::mm::{LOW_MAX_VA, PAGE_SIZE_BITS};

/// User maximum pages
pub const USER_MAX_PAGES: usize = (LOW_MAX_VA + 1) >> PAGE_SIZE_BITS;

/// User heap size
pub const USER_HEAP_SIZE: usize = 0x40_0000;

/// User heap pages
pub const USER_HEAP_PAGES: usize = USER_HEAP_SIZE >> PAGE_SIZE_BITS;

/// User stack size
pub const USER_STACK_SIZE: usize = 0x2_0000;

/// User stack pages
pub const USER_STACK_PAGES: usize = USER_STACK_SIZE >> PAGE_SIZE_BITS;

/// Task stacks starts at the next page of `Trampoline`
pub const USER_STACK_BASE: usize = LOW_MAX_VA + 1;

/// Relocatable file address
pub const ELF_BASE_RELOCATE: usize = 0x8000_0000;