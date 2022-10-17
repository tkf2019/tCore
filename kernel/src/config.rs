/// Boot kernel size allocated in `_start` for single CPU.
pub const BOOT_STACK_SIZE: usize = 0x10000;

/// Support 4 KB page
pub const PAGE_SIZE: usize = 0x1000;

/// One beyond the highest possible virtual address allowed by Sv39.
pub const MAX_VA: usize = 1 << (9 + 9 + 9 + 12 - 1);

/// Trampoline takes up the highest page both in user and kernel space.
pub const TRAMPOLINE_VA: usize = MAX_VA - PAGE_SIZE;

/// TODO! May need to change!
pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;

pub const KERNEL_HEAP_ORDER: usize = 32;
