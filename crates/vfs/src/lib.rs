#![no_std]
#![allow(unused)]

mod flags;
mod link;
mod path;
pub mod ring_buf;
mod stat;

extern crate alloc;

use alloc::{sync::Arc, vec::Vec};
use core::any::Any;
use errno::Errno;

pub use flags::*;
pub use link::*;
pub use path::*;
pub use stat::*;

/// In UNIX, everything is a File, such as:
///
/// 1. A normal file staying on disk.
pub trait File: Send + Sync + AsAny {
    /// Reads bytes from this file to the buffer.
    ///
    /// Returns the number of bytes read from this file.
    /// Returns [`None`] if the file is not readable.
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        None
    }

    /// Writes bytes from the buffer to this file.
    ///
    /// Returns the number of bytes written to this file.
    /// Returns [`None`] if the file is not writable.
    fn write(&self, buf: &[u8]) -> Option<usize> {
        None
    }

    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        false
    }

    /// Clear the [`File`].
    fn clear(&self) {}

    /// Reads all bytes from the file.
    ///
    /// Only the size of real file can be known, so this function is `unsafe`.
    unsafe fn read_all(&self) -> Vec<u8> {
        unimplemented!()
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

    /// Returns if the file is ready to read.
    fn read_ready(&self) -> bool {
        false
    }

    /// Returns if the file is ready to write.
    fn write_ready(&self) -> bool {
        false
    }

    /// Moves the cursor with [`SeekWhence`] flags.
    ///
    ///
    /// See `<https://man7.org/linux/man-pages/man2/lseek.2.html>`.
    fn seek(&self, offset: usize, whence: SeekWhence) -> Option<usize> {
        None
    }

    /// Open flags
    fn open_flags(&self) -> OpenFlags {
        OpenFlags::empty()
    }

    /// Gets file `stat`.
    fn get_stat(&self, stat: *mut Stat) -> bool {
        false
    }

    /// Gets file size.
    fn get_size(&self) -> Option<usize> {
        None
    }

    /// Gets current offset.
    fn get_off(&self) -> usize {
        self.seek(0, SeekWhence::Current).unwrap()
    }

    /// If this file is a directory.
    fn is_dir(&self) -> bool {
        false
    }

    /// If this file is a regular file.
    fn is_reg(&self) -> bool {
        false
    }

    /// Gets the number of hard links.
    fn get_nlink(&self) -> Option<usize> {
        None
    }

    /// Gets the absolute path of this file.
    fn get_path(&self) -> Option<Path> {
        None
    }

    fn is_uintr(&self) -> bool {
        false
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait VFS: Send + Sync {
    /// Opens a file object.
    ///
    /// - `pdir`: Absolute path which must start with '/'.
    /// - `name`: the name of the new file.
    /// - `flags`: Standard [`OpenFlags`].
    /// See `<https://man7.org/linux/man-pages/man2/open.2.html>`.
    fn open(&self, pdir: &Path, name: &str, flags: OpenFlags) -> Result<Arc<dyn File>, Errno>;

    /// Makes a directory.
    ///
    /// - `pdir`: Absolute path which must start with '/'.
    /// - `name`: the name of the new directory.
    fn mkdir(&self, pdir: &Path, name: &str) -> Result<(), Errno>;

    /// Checks for existance.
    fn check(&self, path: &Path) -> bool;

    /// Removes a file.
    fn remove(&self, pdir: &Path, name: &str) -> Result<(), Errno>;
}
