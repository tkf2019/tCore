use vfs::File;

/// Data written to `/dev/null` will always be discarded.
pub struct NullFile;

impl File for NullFile {
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

    fn read(&self, _buf: &mut [u8]) -> Option<usize> {
        Some(0)
    }

    fn write(&self, buf: &[u8]) -> Option<usize> {
        Some(buf.len())
    }

    fn seek(&self, _offset: usize, _whence: vfs::SeekWhence) -> Option<usize> {
        Some(0)
    }
}
