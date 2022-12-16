# tCore

## Crates

- `kernel-sync`: Interrupt-safe Mutex in zCore
- `rust-fatfs`: A third-party fatfs implementation, modified for thread-safety.
- `easy-fs`: Filesystem in `rCore-tutorial`
- `tcache`:
  - `trait CacheUnit`
  - `LRUBlockCache`
- `tmm_addr`:
  - `VirtAddr` and `PhysAddr`
  - `Page` and `Frame`
  - `PageRange` and `FrameRange`
- `tmm_rv`:
  - `PTE`
  - `PageTable`
  - `frame_alloc` and `frame_dealloc` using `buddy system`
- `tsyscall`: Syscall interfaces and types
- `talloc`: `RecycleAllocator`
- `ttimer`
- `tvfs`: `trait File`
- `terrno`: `errno` constants

## Kernel

Syscall implementations:
- Function arguments and return value according to this [blog](https://jborza.com/post/2021-05-11-riscv-linux-syscalls/)

In system programming, we usually use seperate page tables in different privileges to avoid vulnerability caused by Meltdown and Spectre.

Thus a demand occurs as following:

* A syscall handler receives a pointer saved in a register with the type of `usize`.
* The kernel tries to get the buffer starting at this pointer.
* The buffer may be larger than a page, and the contiguous virtual address range may be translated into a disconguous list of physical address ranges.

When I tried to import a third party crate and use the functions, I found that all these functions receives `&[u8]` as a buffer. So I need to convert a list of ranges such as `Vec<&'static mut [u8]>` into a `&[u8]` without performance influenced by Copy of `u8`.
