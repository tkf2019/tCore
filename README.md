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

## Kernel

Syscall implementations:
- Function arguments and return value according to this [blog](https://jborza.com/post/2021-05-11-riscv-linux-syscalls/);
