use crate::{File, SeekWhence};

/// Ring buffer with underlying file.
pub struct RingBuffer<F: File> {
    /// Inner data.
    data: Option<F>,

    /// Head cursor of the buffer.
    head: usize,

    /// Tail cursor of the buffer.
    tail: usize,

    /// Length of data.
    len: usize,

    /// Maximum size.
    max_size: usize,
}

impl<F: File> RingBuffer<F> {
    /// Creates a new ring buffer.
    pub fn new(limit: usize, file: F) -> Self {
        Self {
            data: Some(file),
            head: 0,
            tail: 0,
            len: 0,
            max_size: limit,
        }
    }

    /// Reads data from the buffer as much as possible.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let read_len = buf.len().min(self.len);
        self.len -= read_len;
        let data = self.data.as_ref().unwrap();
        // Read from head
        data.seek(self.head, SeekWhence::Set);
        if self.head + read_len <= self.max_size {
            data.read(&mut buf[..read_len]);
            self.head += read_len;
        } else {
            data.read(&mut buf[..self.max_size - self.head]);
            // Rollback to the start.
            data.seek(0, SeekWhence::Set);
            data.read(&mut buf[self.max_size - self.head..read_len]);
            self.head = self.head + read_len - self.max_size;
        }
        read_len
    }

    /// Writes data to the buffer as much as possible.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        let write_len = buf.len().min(self.max_size - self.len);
        self.len += write_len;
        let data = self.data.as_ref().unwrap();
        // Write to tail
        data.seek(self.tail, SeekWhence::Set);
        if self.tail + write_len <= self.max_size {
            data.write(&buf[..write_len]);
            self.tail += write_len;
        } else {
            data.write(&buf[..self.max_size - self.tail]);
            // Rollback to the start.
            data.seek(0, SeekWhence::Set);
            data.write(&buf[self.max_size - self.tail..write_len]);
            self.tail = self.tail + write_len - self.max_size;
        }
        write_len
    }

    /// Returns true if the buffer has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true of the buffer has a length of size limit.
    pub fn is_full(&self) -> bool {
        self.len == self.max_size
    }
}
