# tCore

## Dependencies

![deps](docs/deps.png)

- `kernel-sync`: Interrupt-safe Mutex in zCore
- `rust-fatfs`: A third-party fatfs implementation, modified for thread-safety.
- `easy-fs`: Filesystem in `rCore-tutorial`
- `device_cache`:
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
- `syscall_interface`: Syscall interfaces and types
- `id_alloc`: `RecycleAllocator`
- `time_subsys`: `TimeSpec` and `TimeVal`
- `vfs`: `trait File`, `Path` to handle
- `errno`: `errno` constants
- `tmemfs`: `MemFile`, `NullFile`, `ZeroFile`
- `tbuffer`: `UserBuffer` for translation, `RingBuffer` for `sys_pipe2`
