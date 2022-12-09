use alloc::{sync::Arc, vec::Vec};
use fatfs::{
    DefaultTimeProvider, FsOptions, IoBase, LossyOemCpConverter, Read, Seek, SeekFrom, Write,
};
use log::{trace, warn};
use spin::{Lazy, Mutex, MutexGuard};
use tcache::{BlockCache, CacheUnit, LRUBlockCache, BLOCK_SIZE};
use terrno::Errno;
use ttimer::TimeSpec;
use tvfs::{File, OpenFlags, Path, SeekWhence, Stat, StatMode, VFS};

use crate::{
    config::{CACHE_SIZE, FS_IMG_SIZE},
    driver::virtio_block::BLOCK_DEVICE,
    error::KernelError,
};

use super::link::get_nlink;

type FatTP = DefaultTimeProvider;
type FatOCC = LossyOemCpConverter;
type FatBlock = [u8; BLOCK_SIZE];
type FatFile = fatfs::File<'static, FatIO, FatTP, FatOCC>;

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

impl IoBase for FatIO {
    type Error = IoError;
}

impl Read for FatIO {
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

impl Write for FatIO {
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

impl Seek for FatIO {
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

/// Mutable data owned by [`Inode`].
pub struct FSFileInner {
    /// Last access.
    pub atime: TimeSpec,

    /// Last modification.
    pub mtime: TimeSpec,

    /// Last change of attributes.
    pub ctime: TimeSpec,

    /// Close-on-exec
    pub cloexec: bool,
}

/// A wrapper for [`FatFile`] to implement [`File`].
///
/// We use three types of regions to maintain the task metadata:
/// - Local and immutable: data initialized once when task created.
/// - Shared and mutable: uses [`Arc<Mutex<T>>`].
/// - Local and mutable: uses [`Mutex<TaskInner>`] to wrap the data together.
pub struct FSFile {
    /// Able to read.
    pub readable: bool,

    /// Able to write.
    pub writable: bool,

    /// Real directory path and file name.
    pub path: Path,

    /// Local and mutable data.
    pub inner: Mutex<FSFileInner>,

    /// Real file in fat.
    pub file: Arc<Mutex<FatFile>>,
}

impl FSFile {
    pub fn new(
        readable: bool,
        writable: bool,
        path: Path,
        file: FatFile,
        flags: OpenFlags,
    ) -> Self {
        Self {
            readable,
            writable,
            path,
            inner: Mutex::new(FSFileInner {
                atime: TimeSpec::default(),
                mtime: TimeSpec::default(),
                ctime: TimeSpec::default(),
                cloexec: flags.contains(OpenFlags::O_CLOEXEC),
            }),
            file: Arc::new(Mutex::new(file)),
        }
    }

    /// Get the length of inner fat file with lock already acquired.
    ///
    /// The cursor will be moved back to current position.
    pub fn get_len(locked: &mut MutexGuard<FatFile>) -> usize {
        let curr_pos = locked.seek(SeekFrom::Current(0)).unwrap();
        let len = locked.seek(SeekFrom::End(0)).unwrap();
        locked.seek(SeekFrom::Start(curr_pos)).unwrap();
        len as usize
    }
}

impl File for FSFile {
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        if !self.readable {
            return None;
        }
        let mut file = self.file.lock();
        let len = buf.len();
        let mut pos = 0;
        while pos < len {
            match file.read(&mut buf[pos..]) {
                Ok(read_len) => {
                    if read_len == 0 {
                        break;
                    } else {
                        pos += read_len;
                    }
                }
                Err(_) => {
                    if pos == 0 {
                        return None;
                    } else {
                        return Some(pos);
                    }
                }
            }
        }
        Some(pos)
    }

    fn write(&self, buf: &[u8]) -> Option<usize> {
        if !self.writable {
            return None;
        }
        let mut file = self.file.lock();
        let len = buf.len();
        let mut pos = 0;
        while pos < len {
            match file.write(&buf[pos..]) {
                Ok(write_len) => {
                    if write_len == 0 {
                        break;
                    } else {
                        pos += write_len;
                    }
                }
                Err(_) => {
                    if pos == 0 {
                        return None;
                    } else {
                        return Some(pos);
                    }
                }
            }
        }
        Some(pos)
    }

    fn clear(&self) {
        let mut file = self.file.lock();
        file.seek(SeekFrom::Start(0)).unwrap();
        file.truncate().unwrap();
    }

    fn seek(&self, offset: usize, whence: SeekWhence) -> Option<usize> {
        let mut file = self.file.lock();
        let seek_from = match whence {
            SeekWhence::Current => SeekFrom::Current(offset as i64),
            SeekWhence::Set => SeekFrom::Start(offset as u64),
            SeekWhence::End => SeekFrom::End(offset as i64),
        };
        let curr_pos = file.seek(SeekFrom::Current(0)).unwrap();
        file.seek(seek_from)
            .map(|pos| match seek_from {
                SeekFrom::Start(offset) => {
                    let len = file.seek(SeekFrom::End(0)).unwrap();
                    if len < offset && offset - len <= FS_IMG_SIZE as u64 {
                        let mut buf: Vec<u8> = Vec::new();
                        buf.resize(offset as usize - len as usize, 0);
                        file.write(buf.as_slice()).unwrap();
                    }
                    file.seek(SeekFrom::Start(offset)).unwrap();
                    Some(offset as usize)
                }
                SeekFrom::Current(offset) => {
                    let len = file.seek(SeekFrom::End(0)).unwrap();
                    let now = (curr_pos as i64 + offset) as u64;
                    if len < now {
                        let mut buf: Vec<u8> = Vec::new();
                        buf.resize(now as usize - len as usize, 0);
                        file.write(buf.as_slice()).unwrap();
                    }
                    file.seek(SeekFrom::Start(now)).unwrap();
                    Some(now as usize)
                }
                SeekFrom::End(_) => Some(pos as usize),
            })
            .unwrap_or_else(|_| {
                trace!("Seek {:?}", seek_from);
                None
            })
    }

    fn get_stat(&self, stat: *mut Stat) -> bool {
        let mut file = self.file.lock();
        let len = FSFile::get_len(&mut file);
        let nlink = get_nlink(&self.path);
        let mode =
            (StatMode::S_IFREG | StatMode::S_IRWXU | StatMode::S_IRWXG | StatMode::S_IRWXO).bits();
        let inner = self.inner.lock();
        unsafe {
            *stat = Stat::new(
                mode,
                nlink as u32,
                len as u64,
                inner.atime,
                inner.mtime,
                inner.ctime,
                BLOCK_SIZE as u32,
            );
        }
        true
    }

    unsafe fn read_all(&self) -> Vec<u8> {
        let mut file = self.file.lock();
        let len = FSFile::get_len(&mut file);
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(len, 0);
        let mut pos = 0;
        while pos < len {
            let read_len = file.read(&mut buf[pos..]).unwrap();
            pos += read_len;
        }
        buf
    }

    fn read_ready(&self) -> bool {
        if !self.readable {
            return false;
        }
        let mut file = self.file.lock();
        let curr_pos = file.seek(SeekFrom::Current(0)).unwrap();
        let len = file.seek(SeekFrom::End(0)).unwrap();
        file.seek(SeekFrom::Start(curr_pos)).unwrap();
        curr_pos < len
    }

    fn write_ready(&self) -> bool {
        if !self.writable {
            return false;
        }
        let mut file = self.file.lock();
        let curr_pos = file.seek(SeekFrom::Current(0)).unwrap();
        let len = file.seek(SeekFrom::End(0)).unwrap();
        file.seek(SeekFrom::Start(curr_pos)).unwrap();
        curr_pos < len
    }
}

/// A wrapper for VFS implementation and configured compilation.
pub struct FileSystem;

/// Global static instance of fat filesystem.
static FAT_FS: Lazy<fatfs::FileSystem<FatIO, FatTP, FatOCC>> = Lazy::new(|| {
    fatfs::FileSystem::new(FatIO::new(), FsOptions::new().update_accessed_date(true)).unwrap()
});

impl VFS for FileSystem {
    fn open(&self, path: &Path, flags: OpenFlags) -> Result<Arc<dyn File>, Errno> {
        let root = FAT_FS.root_dir();
        let raw_path = path.clone();
        let mut path = path.clone();
        let file = match path.pop() {
            Some(file) => file,
            // Empty path is not allowed.
            None => return Err(Errno::EINVAL),
        };
        // Find in the root directory
        let (readable, writable) = flags.read_write();
        let dir = if path.is_root() {
            root
        } else {
            root.open_dir(path.rela()).map_err(|_| Errno::ENOENT)?
        };
        match dir.open_file(file.as_str()) {
            Ok(file) => {
                if flags.contains(OpenFlags::O_CREAT | OpenFlags::O_EXCL) {
                    Err(Errno::EEXIST)
                } else {
                    let file = FSFile::new(readable, writable, path, file, flags);
                    if flags.contains(OpenFlags::O_CREAT) {
                        file.clear();
                    }
                    Ok(Arc::new(file))
                }
            }
            Err(fatfs::Error::NotFound) => {
                // Create if the file not existing
                if flags.contains(OpenFlags::O_CREAT) {
                    let file = dir.create_file(file.as_str()).unwrap();
                    Ok(Arc::new(FSFile::new(
                        readable, writable, raw_path, file, flags,
                    )))
                } else {
                    Err(Errno::ENOENT)
                }
            }
            Err(err) => {
                warn!("{:?}", err);
                Err(Errno::EINVAL)
            }
        }
    }
}
