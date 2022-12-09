use alloc::sync::Arc;
use easy_fs::BlockDevice;
use spin::{Lazy, Mutex};
use tmm_rv::{frame_alloc, frame_dealloc, Frame, PhysAddr, PAGE_SIZE_BITS};
use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};

use crate::mm::KERNEL_MM;

const VIRTIO0: usize = 0x10001000;

pub static BLOCK_DEVICE: Lazy<Arc<dyn BlockDevice>> = Lazy::new(|| {
    Arc::new(unsafe {
        VirtIOBlock(Mutex::new(
            VirtIOBlk::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
        ))
    })
});

pub struct VirtIOBlock(Mutex<VirtIOBlk<'static, VirtioHal>>);

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .lock()
            .read_block(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .lock()
            .write_block(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        frame_alloc(pages).unwrap() << PAGE_SIZE_BITS
    }

    fn dma_dealloc(paddr: usize, pages: usize) -> i32 {
        frame_dealloc(Frame::floor(PhysAddr::from(paddr)).number(), pages);
        0
    }

    fn phys_to_virt(paddr: usize) -> usize {
        paddr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        let pa = KERNEL_MM
            .lock()
            .translate(vaddr.into())
            .expect("Failed to translate virtual address");
        // debug!("virt_to_phys: {:#X} -> {:#?}", vaddr, pa);
        pa.into()
    }
}
