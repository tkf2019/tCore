use crate::SyscallResult;

/// Special value for dirfd.
pub const AT_FDCWD: usize = -100isize as usize;

/// Remove directory instead of unlinking file.
pub const AT_REMOVEDIR: usize = 0x200;

#[repr(C)]
/// Used in readv and writev.
///
/// Defined in sys/uio.h.
pub struct IoVec {
    /// Starting address
    pub iov_base: usize,
    /// Number of bytes to transfer
    pub iov_len: usize,
}

pub trait SyscallFile {
    /// Opens a file.
    ///
    /// If the pathname is relative, then it is interpreted relative
    /// to the directory referred to by the file descriptor dirfd (rather than relative
    /// to the current working directory of the calling process.
    ///
    /// If pathname is relative and dirfd is the special value [`AT_FDCWD`], then pathname
    /// is interpreted relative to the current working directory of the calling process.
    ///
    /// If pathname is absolute, then dirfd is ignored.
    ///
    /// # Argument
    /// - `mode`: The mode argument specifies the file mode bits to be applied when
    /// a new file is created.  If neither O_CREAT nor O_TMPFILE is specified in flags,
    /// then mode is ignored (and can thus be specified as 0, or simply omitted). The
    /// mode argument must be supplied if O_CREAT or O_TMPFILE is specified in flags;
    /// if it is not supplied, some arbitrary bytes from the stack will be applied as
    /// the file mode. The effective mode is modified by the process's umask in the usual way:
    /// in the absence of a default ACL, the mode of the created file is (mode & ~umask).
    ///
    /// # Error
    /// - `EBADF`: pathname is relative but dirfd is neither [`AT_FDCWD`] nor a valid file descriptor.
    /// - `EEXIST`: pathname already exists and O_CREAT and O_EXCL were used.
    /// - `EFAULT`: pathname points outside your accessible address space.
    /// - `EINVAL`: invalid value in flags. O_TMPFILE was specified in flags, but neither O_WRONLY
    /// nor O_RDWR was specified. O_CREAT was specified in flags and the final component  ("basename")
    /// of the new file's pathname is invalid (e.g., it contains characters not permitted by the
    /// underlying filesystem).
    /// - `EISDIR`: pathname refers to a directory and the access requested involved writing (that is,
    /// O_WRONLY or O_RDWR is set).
    /// - `ENOENT`: O_CREAT is not set and the named file does not exist. A directory component
    /// in pathname does not exist or is a dangling symbolic link.
    /// - `ENOTDIR`: A component used as a directory in pathname is not, in fact, a directory,
    /// or O_DIRECTORY was specified and pathname was not a directory. pathname is a relative
    /// pathname and dirfd is a file descriptor referring to a file other than a directory.
    fn openat(dirfd: usize, pathname: *const u8, flags: usize, mode: usize) -> SyscallResult {
        Ok(0)
    }

    /// Close a file descriptor.
    ///
    /// # Error
    /// - `EBADF`: fd isn't a valid open file descriptor.
    fn close(fd: usize) -> SyscallResult {
        Ok(0)
    }

    /// Writes to a file descriptor.
    ///
    ///
    /// # Error
    /// - `EFAULT`: buf is outside your accessible address space.
    /// - `EBADF`: fd is not a valid file descriptor or is not open for writing.
    /// - `EINVAL`: fd is attached to an object which is unsuitable for writing;
    /// or the file was opened with the O_DIRECT flag, and either the address
    /// specified in buf, the value specified in count, or the file offset is
    /// not suitably aligned.
    /// - `EPIPE`: fd is connected to a pipe or socket whose reading end is closed.
    /// When this happens the writing process will also receive a SIGPIPE signal.
    /// (Thus, the write return value is seen only if the program catches, blocks
    /// or ignores this signal.)
    /// - `EINTR`: The call was interrupted by a signal before any data was written;
    /// see signal(7).
    fn write(fd: usize, buf: *const u8, count: usize) -> SyscallResult {
        Ok(0)
    }

    /// Reads from a file descriptor.
    ///
    /// On success, the number of bytes read is returned (zero indicates end of file),
    /// and the file position is advanced by this number. It is not an error if this
    /// number is smaller than the number of bytes requested; this may happen for
    /// example because fewer bytes are actually available right now (maybe because
    /// we were close to end-of-file, or because we are reading from a pipe, or from a
    /// terminal), or because read() was interrupted by a signal.
    ///
    /// # Error
    /// - `EFAULT`: buf is outside your accessible address space.
    /// - `EBADF`: fd is not a valid file descriptor or is not open for reading.
    /// - `EINVAL`: fd is attached to an object which is unsuitable for writing.
    /// or the file was opened with the O_DIRECT flag, and either the address
    /// specified in buf, the value specified in count, or the file offset is
    /// not suitably aligned.
    fn read(fd: usize, buf: *mut u8, count: usize) -> SyscallResult {
        Ok(0)
    }

    /// Repositions the file offset of the open file description associated with
    /// the file descriptor fd to the argument offset according to the directive
    /// whence.
    ///
    /// Upon successful completion, lseek() returns the resulting offset location
    /// as measured in bytes from the beginning of the file. On error, the value
    /// (off_t) -1 is returned and errno is set to indicate the error.
    ///
    /// # Error
    /// - `EBADF`: fd is not an open file descriptor.
    /// - `EINVAL`: whence is not valid.
    /// - `ESPIPE`: fd is associated with a pipe, socket, or FIFO.
    /// - `EOVERFLOW`: The resulting file offset cannot be represented.
    fn lseek(fd: usize, off: usize, whence: usize) -> SyscallResult {
        Ok(0)
    }

    /// Reads `iovcnt` buffers from the file associated with the file descriptor
    /// `fd` into the buffers described by `iov`.
    ///
    /// See [`Self::read`].
    fn readv(fd: usize, iov: *const IoVec, iovcnt: usize) -> SyscallResult {
        Ok(0)
    }

    /// Reads `iovcnt` buffers from the file associated with the file descriptor
    /// `fd` into the buffers described by `iov`.
    ///
    /// See [`Self::write`].
    fn writev(fd: usize, iov: *const IoVec, iovcnt: usize) -> SyscallResult {
        Ok(0)
    }

    /// Deletes a name from the filesystem.  If that name was the last link to a file
    /// and no processes have the file open, the file is deleted and the space it was
    /// using is made available for reuse.
    ///
    /// If the name was the last link to a file but any processes still have the file open,
    /// the file will remain in existence until the last file descriptor referring to it is closed.
    ///
    /// If the pathname is relative, then it is interpreted relative
    /// to the directory referred to by the file descriptor dirfd (rather than relative
    /// to the current working directory of the calling process.
    ///
    /// If pathname is relative and dirfd is the special value [`AT_FDCWD`], then pathname
    /// is interpreted relative to the current working directory of the calling process.
    ///
    /// If pathname is absolute, then dirfd is ignored.
    fn unlinkat(dirfd: usize, pathname: *const u8, flags: usize) -> SyscallResult {
        Ok(0)
    }
}
