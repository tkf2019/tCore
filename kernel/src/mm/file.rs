use alloc::sync::Arc;
use tvfs::File;

/// Memory mapped file.
#[derive(Clone)]
pub struct BackendFile {
    /// Inner file
    file: Arc<dyn File>,

    /// Current offset which indicates where to read or write.
    offset: usize,
}

impl BackendFile {
    /// Creates a new memory mapped file
    pub fn new(file: Arc<dyn File>, offset: usize) -> Self {
        Self { file, offset }
    }

    /// Reads at `off` starting from `self.offset`.
    pub fn read(&self, off: usize, buf: &mut [u8]) -> Option<usize> {
        self.file.read_at_off(off + self.offset, buf)
    }

    /// Writes at `off` starting from `self.offset`.
    pub fn write(&self, off: usize, buf: &[u8]) -> Option<usize> {
        self.file.write_at_off(off + self.offset, buf)
    }

    /// Seeks to `off` starting from `self.offset`.
    pub fn seek(&mut self, off: usize) {
        self.offset += off;
    }

    /// Split at `off` starting from `self.offset`
    pub fn split(&self, off: usize) -> Self {
        Self {
            file: self.file.clone(),
            offset: self.offset + off,
        }
    }
}
