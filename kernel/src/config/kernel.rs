use mm_rv::PAGE_SIZE_BITS;
pub use mm_rv::{LOW_MAX_VA, MAX_VA, PAGE_SIZE};

/// Address alignment
pub const ADDR_ALIGN: usize = core::mem::size_of::<usize>();

/// Use guard page to avoid stack overflow.
pub const GUARD_PAGE: usize = PAGE_SIZE;

/// Trampoline takes up the highest page both in user and kernel space.
pub const TRAMPOLINE_VA: usize = MAX_VA - PAGE_SIZE + 1;

/// CPUs
pub const CPU_NUM: usize = 1;

/// Use cpu0 as main hart
pub const MAIN_HART: usize = 0;

/// Boot kernel size allocated in `_start` for single CPU.
pub const BOOT_STACK_SIZE: usize = 0x4_0000;

/// Total boot kernel size.
pub const TOTAL_BOOT_STACK_SIZE: usize = BOOT_STACK_SIZE * CPU_NUM;

/// Kernel stack size
pub const KERNEL_STACK_SIZE: usize = 0x1_0000;

/// Kernel stack pages
pub const KERNEL_STACK_PAGES: usize = KERNEL_STACK_SIZE >> PAGE_SIZE_BITS;

/// Kernel heap size
pub const KERNEL_HEAP_SIZE: usize = 0x200_0000;

/// Kernel heap pages
pub const KERNEL_HEAP_PAGES: usize = KERNEL_HEAP_SIZE >> PAGE_SIZE_BITS;

/// Used for kernel buddy system allocator
pub const KERNEL_HEAP_ORDER: usize = 32;

/// 256MB physical memory
pub const PHYSICAL_MEMORY_END: usize = 0x9000_0000;

/// VIRTIO base
pub const VIRTIO0: usize = 0x1000_1000;
/// VIRTIO size
pub const VIRTIO_SIZE: usize = 0x1000;

/// MMIO
pub const MMIO: &[(usize, usize)] = &[
    (VIRTIO0, VIRTIO_SIZE),   // Virtio Block in virt machine
];

/// The number of block cache units for virtio.
pub const CACHE_SIZE: usize = 32;

/// Size of virtual block device: 40 MB
pub const FS_IMG_SIZE: usize = 40 * 1024 * 1024;

/// Default maximum file descriptor limit.
pub const DEFAULT_FD_LIMIT: usize = 0x100;

/// Boot root directory
pub const ROOT_DIR: &str = "/";

/// Absolute path of init task
pub const INIT_TASK_PATH: &str = "hello_world";

/// TEST
cfg_if::cfg_if! {
    if #[cfg(feature = "test")] {
        pub const IS_TEST_ENV: bool = true;
    } else {
        pub const IS_TEST_ENV: bool = false;
    }
}

/// Maximum virtual memory areas in an address space
pub const MAX_MAP_COUNT: usize = 256;

/// Maximum size of  pipe buffer.
pub const MAX_PIPE_BUF: usize = PAGE_SIZE;

/// Timer interrupt per second
pub const INTR_PER_SEC: usize = 10;