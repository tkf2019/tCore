use vfs::File;

/// Data written to `/dev/zero` will always be discarded.
/// Buffer will be filled with zero after reading from it.
pub struct ZeroFile;

impl File for ZeroFile {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn read_ready(&self) -> bool {
        true
    }

    fn write_ready(&self) -> bool {
        true
    }

    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        buf.fill(0);
        Some(buf.len())
    }

    fn write(&self, buf: &[u8]) -> Option<usize> {
        Some(buf.len())
    }

    fn seek(&self, _offset: usize, _whence: vfs::SeekWhence) -> Option<usize> {
        Some(0)
    }
}
