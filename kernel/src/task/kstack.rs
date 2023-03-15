use id_alloc::*;
use kernel_sync::SpinLock;
use spin::Lazy;

use crate::{
    config::*,
    error::KernelResult,
    mm::{VMFlags, KERNEL_MM},
};

/// Global kernal stack allocator.
static KSTACK_ALLOCATOR: Lazy<SpinLock<RecycleAllocator>> =
    Lazy::new(|| SpinLock::new(RecycleAllocator::new(1)));

/// Allocate new kernel stack identification.
pub fn kstack_alloc() -> usize {
    KSTACK_ALLOCATOR.lock().alloc()
}

/// Deallocate kernel stack by identification.
pub fn kstack_dealloc(kstack: usize) {
    KSTACK_ALLOCATOR.lock().dealloc(kstack)
}

/// Returns kernel stack layout [top, base) by kernel stack identification.
///
/// Stack grows from high address to low address.
pub fn kstack_layout(kstack: usize) -> (usize, usize) {
    let base = TRAMPOLINE_VA - kstack * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let top = base - KERNEL_STACK_SIZE;
    (top, base - ADDR_ALIGN)
}

/// Allocate a kernel stack for the task by kernel stack identification.
///
/// Returns the kernel stack base.
pub fn kstack_vm_alloc(kstack: usize) -> KernelResult<usize> {
    let (kstack_top, kstack_base) = kstack_layout(kstack);
    KERNEL_MM.lock().alloc_write_vma(
        None,
        kstack_top.into(),
        kstack_base.into(),
        VMFlags::READ | VMFlags::WRITE,
    )?;
    Ok(kstack_base)
}
