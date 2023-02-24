# Rust kernel-sync

Kernel synchronization primitives implemented in Rust:

- [ ] Local interrupt disabling: Forbid interrupt handling on a single CPU.
- [ ] Spin Lock: Lock with busy wait.
- [ ] Semaphore: Lock with blocking wait (sleep).
- [ ] Read-Copy-Update (RCU): Lock-free access to shared data structures through pointers.

Features:

- Interrupt dependent on architure:
  - [ ] riscv64

## Usage

