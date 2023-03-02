use alloc::{sync::Arc, vec::Vec};
use core::cell::SyncUnsafeCell;
use device_cache::{BlockCache, CacheUnit, LRUBlockCache, BLOCK_SIZE};
use errno::Errno;
use fatfs::{
    DefaultTimeProvider, FsOptions, IoBase, LossyOemCpConverter, Read, Seek, SeekFrom, Write,
};
use kernel_sync::SpinLock;
use log::{trace, warn};
use spin::Lazy;
use time_subsys::TimeSpec;
use vfs::*;

use crate::{
    config::{CACHE_SIZE, FS_IMG_SIZE},
    driver::virtio_block::BLOCK_DEVICE,
    error::KernelError,
};

type FatTP = DefaultTimeProvider;
type FatOCC = LossyOemCpConverter;
type FatBlock = [u8; BLOCK_SIZE];
type FatFile = fatfs::File<'static, FatIO, FatTP, FatOCC>;
type FatDir = fatfs::Dir<'static, FatIO, FatTP, FatOCC>;

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

fn from(value: fatfs::Error<IoError>) -> Errno {
    match value {
        fatfs::Error::NotFound => Errno::ENOENT,
        fatfs::Error::AlreadyExists => Errno::EEXIST,
        fatfs::Error::InvalidFileNameLength => Errno::ENAMETOOLONG,
        _ => Errno::EINVAL,
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

/// A wrapper for [`FatFile`] to implement [`File`].
///
/// We use three types of regions to maintain the task metadata:
/// - Local and immutable: data initialized once when task created.
/// - Shared and mutable: uses [`Arc<SpinLock<T>>`].
/// - Local and mutable: uses [`SpinLock<TaskInner>`] to wrap the data together.
pub struct File {
    /// Able to read.
    pub readable: bool,

    /// Able to write.
    pub writable: bool,

    /// Real directory path and file name.
    pub path: Path,

    /// Local and mutable data.
    pub inner: SpinLock<FSFileInner>,

    /// Real file in fat.
    pub file: SyncUnsafeCell<FatFile>,
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
            inner: SpinLock::new(FSFileInner {
                atime: TimeSpec::default(),
                mtime: TimeSpec::default(),
                ctime: TimeSpec::default(),
                cloexec: flags.contains(OpenFlags::O_CLOEXEC),
            }),
            file: SyncUnsafeCell::new(file),
        }
    }

    /// Gets the raw mutable reference to inner file without any borrow check.
    pub fn file(&self) -> &'static mut FatFile {
        unsafe { &mut *self.file.get() }
    }
}

impl Drop for FSFile {
    fn drop(&mut self) {
        trace!("Drop FSfile");
        // Flush the file to disk manually.
        let _guard = GLOBAL_FS.lock();
        if let Err(err) = self.file().flush() {
            warn!("flush failed {:?}", err);
        }
        drop(_guard);
    }
}

impl File for FSFile {
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        trace!("FSFile::read");
        if !self.readable {
            return None;
        }
        let len = buf.len();
        let mut pos = 0;
        while pos < len {
            let _guard = GLOBAL_FS.lock();
            match self.file().read(&mut buf[pos..]) {
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
            drop(_guard);
        }
        Some(pos)
    }

    fn write(&self, buf: &[u8]) -> Option<usize> {
        trace!("FSFile::write");
        if !self.writable {
            return None;
        }
        let len = buf.len();
        let mut pos = 0;
        while pos < len {
            let _guard = GLOBAL_FS.lock();
            match self.file().write(&buf[pos..]) {
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
            drop(_guard);
        }
        Some(pos)
    }

    fn readable(&self) -> bool {
        self.readable
    }

    fn writable(&self) -> bool {
        self.writable
    }

    #[no_mangle]
    fn clear(&self) {
        trace!("FSFile::clear");
        let _guard = GLOBAL_FS.lock();
        self.file().seek(SeekFrom::Start(0)).unwrap();
        self.file().truncate().unwrap();
        drop(_guard);
    }

    fn seek(&self, offset: usize, whence: SeekWhence) -> Option<usize> {
        let seek_from = match whence {
            SeekWhence::Current => SeekFrom::Current(offset as i64),
            SeekWhence::Set => SeekFrom::Start(offset as u64),
            SeekWhence::End => SeekFrom::End(offset as i64),
        };
        let _guard = GLOBAL_FS.lock();
        let curr_pos = self.file().seek(SeekFrom::Current(0)).unwrap();
        let result = self
            .file()
            .seek(seek_from)
            .map(|pos| match seek_from {
                SeekFrom::Start(offset) => {
                    let len = self.file().seek(SeekFrom::End(0)).unwrap();
                    if len < offset && offset <= FS_IMG_SIZE as u64 {
                        let mut buf: Vec<u8> = Vec::new();
                        buf.resize(offset as usize - len as usize, 0);
                        self.file().write(buf.as_slice()).unwrap();
                    }
                    self.file().seek(SeekFrom::Start(offset)).unwrap();
                    Some(offset as usize)
                }
                SeekFrom::Current(offset) => {
                    let len = self.file().seek(SeekFrom::End(0)).unwrap();
                    let now = (curr_pos as i64 + offset) as u64;
                    if len < now && now <= FS_IMG_SIZE as u64 {
                        let mut buf: Vec<u8> = Vec::new();
                        buf.resize(now as usize - len as usize, 0);
                        self.file().write(buf.as_slice()).unwrap();
                    }
                    self.file().seek(SeekFrom::Start(now)).unwrap();
                    Some(now as usize)
                }
                SeekFrom::End(_) => Some(pos as usize),
            })
            .unwrap_or_else(|_| {
                trace!("Seek {:?}", seek_from);
                None
            });
        drop(_guard);
        result
    }

    fn get_stat(&self, stat_ptr: *mut Stat) -> bool {
        let mut stat = Stat::default();
        stat.st_mode =
            (StatMode::S_IFREG | StatMode::S_IRWXU | StatMode::S_IRWXG | StatMode::S_IRWXO).bits();
        stat.st_nlink = get_nlink(&self.path) as u32;

        let _guard = GLOBAL_FS.lock();
        stat.st_size = self.get_size().unwrap() as u64;
        drop(_guard);

        let inner = self.inner.lock();
        stat.st_blksize = BLOCK_SIZE as u32;
        stat.st_blocks = (stat.st_size + stat.st_blksize as u64 - 1) / stat.st_blksize as u64;
        stat.st_atime_sec = inner.atime.tv_sec;
        stat.st_atime_sec = inner.atime.tv_nsec;
        stat.st_mtime_sec = inner.mtime.tv_sec;
        stat.st_mtime_sec = inner.mtime.tv_nsec;
        stat.st_ctime_sec = inner.ctime.tv_sec;
        stat.st_ctime_sec = inner.ctime.tv_nsec;
        unsafe { *stat_ptr = stat };
        true
    }

    unsafe fn read_all(&self) -> Vec<u8> {
        let _guard = GLOBAL_FS.lock();
        let len = self.get_size().unwrap();
        trace!("FSFile::read_all 0x{:x}", len);
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(len, 0);
        let mut pos = 0;
        while pos < len {
            let read_len = self.file().read(&mut buf[pos..]).unwrap();
            pos += read_len;
        }
        drop(_guard);
        buf
    }

    fn read_ready(&self) -> bool {
        if !self.readable {
            return false;
        }
        let _guard = GLOBAL_FS.lock();
        let curr_pos = self.file().seek(SeekFrom::Current(0)).unwrap();
        let len = self.file().seek(SeekFrom::End(0)).unwrap();
        self.file().seek(SeekFrom::Start(curr_pos)).unwrap();
        drop(_guard);
        curr_pos < len
    }

    fn write_ready(&self) -> bool {
        if !self.writable {
            return false;
        }
        let _guard = GLOBAL_FS.lock();
        let curr_pos = self.file().seek(SeekFrom::Current(0)).unwrap();
        let len = self.file().seek(SeekFrom::End(0)).unwrap();
        self.file().seek(SeekFrom::Start(curr_pos)).unwrap();
        drop(_guard);
        curr_pos < len
    }

    fn is_reg(&self) -> bool {
        true
    }

    fn get_path(&self) -> Option<Path> {
        Some(self.path.clone())
    }

    fn get_size(&self) -> Option<usize> {
        let curr_pos = self.file().seek(SeekFrom::Current(0)).unwrap();
        let len = self.file().seek(SeekFrom::End(0)).unwrap();
        self.file().seek(SeekFrom::Start(curr_pos)).unwrap();
        Some(len as usize)
    }
}

/// A wrapper for directory path to implement [`File`].
pub struct FSDir {
    /// Real directory path.
    pub path: Path,
}

impl FSDir {
    pub fn new(path: Path) -> Self {
        Self { path }
    }
}

impl File for FSDir {
    fn get_path(&self) -> Option<Path> {
        Some(self.path.clone())
    }

    fn is_dir(&self) -> bool {
        true
    }
}

/// A wrapper for VFS implementation and configured compilation.
pub struct FileSystem;

impl Drop for FileSystem {
    fn drop(&mut self) {
        let _guard = GLOBAL_FS.lock();
        if let Err(err) = FAT_FS.unmount_internal() {
            warn!("unmount failed {:?}", err);
        }
        drop(_guard);
    }
}

/// Global disk filesystem.
///
/// TODO: A big lock on the filesystem!
pub static GLOBAL_FS: Lazy<SpinLock<FileSystem>> = Lazy::new(|| {
    let fs = FileSystem;

    let root = Path::root();
    fs.mkdir(&root, "dev").unwrap();
    fs.mkdir(&root, "lib").unwrap();
    fs.mkdir(&root, "tmp").unwrap();

    SpinLock::new(fs)
});

/// Global static instance of fat filesystem.
static FAT_FS: Lazy<fatfs::FileSystem<FatIO, FatTP, FatOCC>> = Lazy::new(|| {
    fatfs::FileSystem::new(FatIO::new(), FsOptions::new().update_accessed_date(true)).unwrap()
});

impl VFS for FileSystem {
    fn open(&self, pdir: &Path, name: &str, flags: OpenFlags) -> Result<Arc<dyn File>, Errno> {
        let mut ori_path = pdir.clone();
        ori_path.extend(name);
        trace!("FileSystem::open {:x?}", ori_path);

        let (readable, writable) = flags.read_write();
        let root = FAT_FS.root_dir();
        // Find in the root directory
        let pdir = if pdir.is_root() {
            root
        } else {
            root.open_dir(pdir.rela()).map_err(|_| Errno::ENOENT)?
        };

        if flags.contains(OpenFlags::O_DIRECTORY | OpenFlags::O_DSYNC) || ori_path.is_dir() {
            match pdir.open_dir(name) {
                Ok(_) => Ok(Arc::new(FSDir::new(ori_path))),
                Err(err) => Err(from(err)),
            }
        } else {
            match pdir.open_file(name) {
                Ok(file) => {
                    if flags.contains(OpenFlags::O_CREAT | OpenFlags::O_EXCL) {
                        Err(Errno::EEXIST)
                    } else {
                        let file = FSFile::new(readable, writable, ori_path, file, flags);
                        if flags.contains(OpenFlags::O_CREAT) {
                            file.clear();
                        }
                        Ok(Arc::new(file))
                    }
                }
                Err(fatfs::Error::NotFound) => {
                    // Create if the file not existing
                    if flags.contains(OpenFlags::O_CREAT) {
                        let file = pdir.create_file(name).unwrap();
                        Ok(Arc::new(FSFile::new(
                            readable, writable, ori_path, file, flags,
                        )))
                    } else {
                        Err(Errno::ENOENT)
                    }
                }
                Err(err) => Err(from(err)),
            }
        }
    }

    fn mkdir(&self, pdir: &Path, name: &str) -> Result<(), Errno> {
        let mut ori_path = pdir.clone();
        ori_path.extend(name);
        let root = FAT_FS.root_dir();
        let pdir = if pdir.is_root() {
            root
        } else {
            root.open_dir(pdir.rela()).map_err(|_| Errno::ENOENT)?
        };
        for entry in pdir.iter() {
            if entry.unwrap().file_name() == name {
                return Err(Errno::EEXIST);
            }
        }
        pdir.create_dir(name).map_err(|err| from(err))?;
        Ok(())
    }

    fn check(&self, path: &Path) -> bool {
        let root = FAT_FS.root_dir();
        if path.is_dir() {
            if path.is_root() {
                return true;
            }
            root.open_dir(path.rela()).is_ok()
        } else {
            root.open_file(path.rela()).is_ok()
        }
    }

    fn remove(&self, pdir: &Path, name: &str) -> Result<(), Errno> {
        let root = FAT_FS.root_dir();
        let pdir = if pdir.is_root() {
            root
        } else {
            root.open_dir(pdir.rela()).map_err(|_| Errno::ENOENT)?
        };
        pdir.remove(name).map_err(|err| from(err))
    }
}
