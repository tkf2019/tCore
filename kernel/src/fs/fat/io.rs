use device_cache::{BlockCache, CacheUnit, LRUBlockCache, BLOCK_SIZE};
use errno::Errno;
use fatfs::SeekFrom;

use crate::{
    config::{CACHE_SIZE, FS_IMG_SIZE},
    driver::virtio_block::BLOCK_DEVICE,
    error::KernelError,
};

type FatBlock = [u8; BLOCK_SIZE];

/// IO wrapper for FAT.
pub struct FatIO {
    /// Inner block cache.
    pub cache: LRUBlockCache,

    /// Can move within the range of memory mapped block device for `Seek` operation.
    ///
    /// Attention: `pos` is the offset from the start.
    pub pos: usize,

    /// Maximum size of the block device.
    pub max_size: usize,
}

impl FatIO {
    /// Create a new wrapper.
    pub fn new() -> Self {
        Self {
            cache: LRUBlockCache::new(CACHE_SIZE),
            pos: 0,
            max_size: FS_IMG_SIZE,
        }
    }
}

#[derive(Debug)]
pub struct IoError(KernelError);

impl fatfs::IoError for IoError {
    fn is_interrupted(&self) -> bool {
        self.0 == KernelError::IOInterrupted
    }

    fn new_unexpected_eof_error() -> Self {
        Self(KernelError::IOUnexpectedEof)
    }

    fn new_write_zero_error() -> Self {
        Self(KernelError::IOWriteZero)
    }
}

pub fn from(value: fatfs::Error<IoError>) -> Errno {
    match value {
        fatfs::Error::NotFound => Errno::ENOENT,
        fatfs::Error::AlreadyExists => Errno::EEXIST,
        fatfs::Error::InvalidFileNameLength => Errno::ENAMETOOLONG,
        _ => Errno::EINVAL,
    }
}

impl fatfs::IoBase for FatIO {
    type Error = IoError;
}

impl fatfs::Read for FatIO {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let len = core::cmp::min(self.max_size - self.pos, buf.len());
        let start_id = self.pos / BLOCK_SIZE;
        let end_id = (self.pos + len - 1) / BLOCK_SIZE;
        let mut block_ptr = self.pos;
        let mut buf_ptr = 0;
        let mut rem_len = len;
        for block_id in start_id..=end_id {
            let block_off = block_ptr % BLOCK_SIZE;
            let read_len = if block_id == end_id {
                rem_len
            } else {
                BLOCK_SIZE - block_off
            };
            self.cache
                .get_block(block_id, BLOCK_DEVICE.clone())
                .lock()
                .read(0, |block: &FatBlock| {
                    (&mut buf[buf_ptr..buf_ptr + read_len])
                        .copy_from_slice(&block[block_off..block_off + read_len])
                });
            block_ptr += read_len;
            buf_ptr += read_len;
            rem_len -= read_len;
        }
        assert_eq!(rem_len, 0);
        self.pos = block_ptr;
        Ok(len)
    }
}

impl fatfs::Write for FatIO {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let len = core::cmp::min(self.max_size - self.pos, buf.len());
        let start_id = self.pos / BLOCK_SIZE;
        let end_id = (self.pos + len - 1) / BLOCK_SIZE;
        let mut block_ptr = self.pos;
        let mut buf_ptr = 0;
        let mut rem_len = len;
        for block_id in start_id..=end_id {
            let block_off = block_ptr % BLOCK_SIZE;
            let write_len = if block_id == end_id {
                rem_len
            } else {
                BLOCK_SIZE - block_off
            };
            self.cache
                .get_block(block_id, BLOCK_DEVICE.clone())
                .lock()
                .write(0, |block: &mut FatBlock| {
                    (&mut block[block_off..block_off + write_len])
                        .copy_from_slice(&buf[buf_ptr..buf_ptr + write_len])
                });
            block_ptr += write_len;
            buf_ptr += write_len;
            rem_len -= write_len;
        }
        assert_eq!(rem_len, 0);
        self.pos = block_ptr;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // The kernel might crash before sync finished.
        self.cache.sync_all();
        Ok(())
    }
}

impl fatfs::Seek for FatIO {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let new_pos = match pos {
            SeekFrom::Current(delta) => (self.pos as i64 + delta) as usize,
            SeekFrom::Start(delta) => delta as usize,
            SeekFrom::End(delta) => (self.max_size as i64 + delta) as usize,
        };
        if new_pos > self.max_size {
            Err(IoError(KernelError::IOUnexpectedEof))
        } else {
            self.pos = new_pos;
            Ok(self.pos as u64)
        }
    }
}
