#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String, vec::Vec};

use crate::SuperBlock;

bitflags::bitflags! {
    pub struct OpenFlags: u32 {
        /// Read only
        const O_RDONLY = 0o0;

        /// Write only
        const O_WRONLY = 0o1;

        /// Read / Write
        const O_RDWR = 0o2;

        /// Create if
        const O_CREAT = 0o100;

        /// Ensure that this call creates the file: if this flag is specified in conjunction
        /// with O_CREAT, and pathname already exists, then open() fails with the error EEXIST.
        const O_EXCL = 0o200;

        /// If pathname refers to a terminal device—see tty(4)—it will not become the process's
        /// controlling terminal even if the process does not have one.
        const O_NOCTTY = 0o400;

        /// If the file already exists and is a regular file and the access mode allows writing
        /// (i.e., is O_RDWR or O_WRONLY) it will be truncated to length 0. If the file is a FIFO
        /// or terminal device file, the O_TRUNC flag is ignored. Otherwise, the effect of O_TRUNC
        /// is unspecified.
        const O_TRUNC = 0o1000;

        /// The file is opened in append mode. Before each write(2), the file offset
        /// is positioned at the end of the file, as if with lseek(2). The modification
        /// of the file offset and the write operation are performed as a single atomic step.
        const O_APPEND = 0o2000;

        /// When possible, the file is opened in nonblocking mode. Neither the open()
        /// nor any subsequent I/O operations on the file descriptor which is returned
        /// will cause the calling process to wait.
        const O_NONBLOCK = 0o4000;

        /// Write operations on the file will complete according to the requirements of
        /// synchronized I/O data integrity completion.
        const O_DSYNC = 0o200000;

        /// If pathname is not a directory, cause the open to fail.
        const O_DIRECTORY = 0o200000;

        /// If the trailing component (i.e., basename) of pathname is a symbolic link,
        /// then the open fails, with the error ELOOP.  Symbolic links in earlier
        /// components of the pathname will still be followed.  (Note that the ELOOP
        /// error that can occur in this case is indistinguishable from the case where
        /// an open fails because there are too many symbolic links found while resolving
        ///  components in the prefix part of the pathname.)
        const O_NOFOLLOW = 0o400000;

        /// Close-on-exec
        const O_CLOEXEC = 0o2000000;

        /// Obtain a file descriptor that can be used for two purposes: to indicate a
        /// location in the filesystem tree and to perform operations that act purely at
        /// the file descriptor level. The file itself is not opened, and other file
        /// operations (e.g., read(2), write(2), fchmod(2), fchown(2), fgetxattr(2),
        /// ioctl(2), mmap(2)) fail with the error EBADF.
        ///
        /// When O_PATH is specified in flags, flag bits other than `O_CLOEXEC`, `O_DIRECTORY`,
        /// and `O_NOFOLLOW` are ignored.
        const O_PATH = 0o10000000;
    }
}

impl OpenFlags {
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::O_WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }

    pub fn writable(&self) -> bool {
        self.contains(Self::O_WRONLY) || self.contains(Self::O_RDWR)
    }

    pub fn readable(&self) -> bool {
        !self.contains(Self::O_WRONLY)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekWhence {
    /// Sets the offset to the provided number of bytes.
    Set = 0,

    /// Sets the offset to the current position plus the specified number of bytes.
    Current = 1,

    /// Sets the offset to the size of this object plus the specified number of bytes.
    End = 2,
}

pub struct File {
    /// Flags specified when opening the file.
    flags: OpenFlags,

    /// Pointer to file operation table.
    ops: Box<dyn FileOperations>,

    /// Current file offset (file pointer).
    off: SpinLock<usize>,

    /// Pointer to superblock object.
    sb: &mut SuperBlock,
}

pub trait FileOperations: Send + Sync {
    /// Reads bytes from this file to the buffer.
    ///
    /// Returns the number of bytes read from this file.
    /// Returns [`None`] if the file is not readable.
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        None
    }

    /// Reads the file starting at offset to buffer.
    ///
    /// Returns the number bytes read successfully.
    fn read_at_off(&self, off: usize, buf: &mut [u8]) -> Option<usize> {
        let curr_pos = self.seek(0, SeekWhence::Current)?;
        self.seek(off, SeekWhence::Set)?;
        let read_len = self.read(buf);
        self.seek(curr_pos, SeekWhence::Set)?;
        read_len
    }

    /// Reads all bytes from the file.
    ///
    /// Only the size of real file can be known, so this function is `unsafe`.
    unsafe fn read_all(&self) -> Vec<u8> {
        unimplemented!()
    }

    /// Writes bytes from the buffer to this file.
    ///
    /// Returns the number of bytes written to this file.
    /// Returns [`None`] if the file is not writable.
    fn write(&self, buf: &[u8]) -> Option<usize> {
        None
    }

    /// Writes the file starting at offset from buffer.
    ///
    /// Returns the number of bytes written successfully.
    fn write_at_off(&self, off: usize, buf: &[u8]) -> Option<usize> {
        let curr_pos = self.seek(0, SeekWhence::Current)?;
        self.seek(off, SeekWhence::Set)?;
        let write_len = self.write(buf);
        self.seek(curr_pos, SeekWhence::Set)?;
        write_len
    }

    /// Moves the cursor with [`SeekWhence`] flags.
    ///
    /// See `<https://man7.org/linux/man-pages/man2/lseek.2.html>`.
    fn seek(&self, offset: usize, whence: SeekWhence) -> Option<usize> {
        None
    }

    /// Called when reference to an open file is closed.
    fn flush(&self) {}

    /// Flushes the file by writing all cached data to disk.
    fn fsync(&self) {}
}
