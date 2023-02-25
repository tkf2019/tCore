use alloc::sync::Arc;
use kernel_sync::SpinLock;
use vfs::{ring_buf::RingBuffer, File};

use crate::{config::MAX_PIPE_BUF, fs::mem::MemFile, task::do_yield};

pub struct Pipe {
    /// If this is a read end of pipe.
    is_read: bool,

    /// Inner data in a ring buffer.
    buf: Arc<SpinLock<RingBuffer<MemFile>>>,
}

impl Pipe {
    /// Creates a read end and a wirte end of a pipe at the smae time.
    pub fn new() -> (Self, Self) {
        let buf = Arc::new(SpinLock::new(RingBuffer::new(
            MAX_PIPE_BUF,
            MemFile::new(MAX_PIPE_BUF),
        )));
        (
            Self {
                is_read: true,
                buf: buf.clone(),
            },
            Self {
                is_read: false,
                buf,
            },
        )
    }
}

impl File for Pipe {
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        if !self.is_read {
            return None;
        }

        let mut read_len = 0;
        loop {
            let buf_rc = Arc::strong_count(&self.buf);
            let mut ring_buf = self.buf.lock();
            if ring_buf.is_empty() {
                // Write end closed.
                if buf_rc == 1 {
                    return Some(0);
                }
                // Release the lock.
                drop(ring_buf);
                do_yield();
                continue;
            }
            read_len += ring_buf.read(&mut buf[read_len..]);
            break;
        }
        Some(read_len)
    }

    fn write(&self, buf: &[u8]) -> Option<usize> {
        if self.is_read {
            return None;
        }

        let mut write_len = 0;
        loop {
            let buf_rc = Arc::strong_count(&self.buf);
            let mut ring_buf = self.buf.lock();
            if ring_buf.is_full() {
                // Read end closed.
                if buf_rc == 1 {
                    // TODO: raise SIGPIPE and fail with EPIPE
                    return Some(0);
                }
                // Release the lock.
                drop(ring_buf);
                do_yield();
                continue;
            }
            write_len += ring_buf.write(&buf[write_len..]);
            break;
        }
        Some(write_len)
    }

    fn readable(&self) -> bool {
        self.is_read
    }

    fn writable(&self) -> bool {
        !self.is_read
    }

    fn read_ready(&self) -> bool {
        self.is_read && !self.buf.lock().is_empty()
    }

    fn write_ready(&self) -> bool {
        !self.is_read && !self.buf.lock().is_full()
    }

    fn get_off(&self) -> usize {
        0
    }
}
