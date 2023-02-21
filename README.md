# tCore

## Dependencies

![deps](docs/deps.png)

- `kernel-sync`: Interrupt-safe Mutex in zCore
- `rust-fatfs`: A third-party fatfs implementation, modified for thread-safety.
- `easy-fs`: Filesystem in `rCore-tutorial`
- `device-cache`:
  - `trait CacheUnit`
  - `LRUBlockCache`
- `mm-addr`:
  - `VirtAddr` and `PhysAddr`
  - `Page` and `Frame`
  - `PageRange` and `FrameRange`
- `mm-rv`:
  - `PTE`
  - `PageTable`
  - `frame_alloc` and `frame_dealloc` using `buddy system`
- `syscall-interface`: Syscall interfaces and types
- `id-alloc`: `RecycleAllocator`
- `time-subsys`: `TimeSpec` and `TimeVal`
- `vfs`: `trait File`, `Path` to handle
- `errno`: `errno` constants
