use buddy_system_allocator::LockedHeap;
use log::error;

use super::config::{KERNEL_HEAP_ORDER, KERNEL_HEAP_SIZE};

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<KERNEL_HEAP_ORDER> = LockedHeap::<KERNEL_HEAP_ORDER>::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    error!("[kernel] Heap allocation error: {:?}", layout);
    panic!()
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}
