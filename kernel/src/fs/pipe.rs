use alloc::sync::Arc;
use spin::Mutex;
use tbuffer::RingBuffer;
use tmemfs::MemFile;

pub struct Pipe {
    /// If this is a read end of pipe.
    is_read: bool,

    /// Inner data in a ring buffer.
    data: Arc<Mutex<RingBuffer<MemFile>>>,
}
