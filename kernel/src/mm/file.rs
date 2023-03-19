use alloc::sync::Arc;
use vfs::File;

use super::MmapProt;

/// Memory mapped file.
#[derive(Clone)]
pub struct MmapFile {
    /// Inner file
    file: Arc<dyn File>,

    /// Current offset which indicates where to read or write.
    offset: usize,
}

impl MmapFile {
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

    /// Split at `off` starting from `self.offset`
    pub fn split(&self, off: usize) -> Self {
        Self {
            file: self.file.clone(),
            offset: self.offset + off,
        }
    }

    /// Checks the given access flags.
    pub fn mprot(&self, prot: MmapProt) -> bool {
        (self.file.readable() || !prot.contains(MmapProt::PROT_READ))
            && (self.file.writable() || !prot.contains(MmapProt::PROT_WRITE))
    }
}
