/*!
- PAGE_SIZE is the minimum unit of user address space areas, so user
heap and stack have better be integral multiple of PAGE_SIZE.
*/
#![allow(unused)]

use tmm_rv::PAGE_SIZE_BITS;
pub use tmm_rv::{LOW_MAX_VA, MAX_VA, PAGE_SIZE};

pub const ADDR_ALIGN: usize = core::mem::size_of::<usize>();

/* Global configurations */

/// Use guard page to avoid stack overflow.
pub const GUARD_PAGE: usize = PAGE_SIZE;

/// Trampoline takes up the highest page both in user and kernel space.
pub const TRAMPOLINE_VA: usize = MAX_VA - PAGE_SIZE + 1;

/* Kernel configurations */

/// CPUs
pub const CPU_NUM: usize = 4;

/// Boot kernel size allocated in `_start` for single CPU.
pub const BOOT_STACK_SIZE: usize = 0x4_0000;

/// Total boot kernel size.
pub const TOTAL_BOOT_STACK_SIZE: usize = BOOT_STACK_SIZE * CPU_NUM;

/// Kernel stack size
pub const KERNEL_STACK_SIZE: usize = 0x1_0000;

/// Kernel stack pages
pub const KERNEL_STACK_PAGES: usize = KERNEL_STACK_SIZE >> PAGE_SIZE_BITS;

/// Kernel heap size
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

/// Kernel heap pages
pub const KERNEL_HEAP_PAGES: usize = KERNEL_HEAP_SIZE >> PAGE_SIZE_BITS;

/// Used for kernel buddy system allocator
pub const KERNEL_HEAP_ORDER: usize = 32;

/// 256MB physical memory
pub const PHYSICAL_MEMORY_END: usize = 0x9000_0000;

/// MMIO
pub const MMIO: &[(usize, usize)] = &[
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
];

/// Main task in the same address space
pub const MAIN_TASK: usize = 0;

/// Use cpu0 as main hart
pub const MAIN_HART: usize = 0;

/// The number of block cache units for virtio.
pub const CACHE_SIZE: usize = 32;

/// Size of virtual block device: 40 MB
pub const FS_IMG_SIZE: usize = 40 * 1024 * 1024;

/// Default maximum file descriptor limit.
pub const DEFAULT_FD_LIMIT: usize = 0x100;

/// ROOT
pub const ROOT_DIR: &str = "/";

/// Absolute path of init task
pub const INIT_TASK_PATH: &str = "rcore/hello_world";

/// TEST
pub const IS_TEST_ENV: bool = true;

/* User configurations */

/// User heap size
pub const USER_HEAP_SIZE: usize = 0x40_0000;

/// User heap pages
pub const USER_HEAP_PAGES: usize = USER_HEAP_SIZE >> PAGE_SIZE_BITS;

/// User stack size
pub const USER_STACK_SIZE: usize = 0x2000;

/// User stack pages
pub const USER_STACK_PAGES: usize = USER_STACK_SIZE >> PAGE_SIZE_BITS;

/// Task stacks starts at the next page of `Trampoline`
pub const USER_STACK_BASE: usize = LOW_MAX_VA + 1;

/// End virtual address of `mmap` area
pub const USER_MMAP_END: usize = LOW_MAX_VA - USER_STACK_SIZE;
