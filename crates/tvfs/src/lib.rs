#![no_std]

#[cfg(test)]
mod test;

use bitflags::bitflags;
use downcast_rs::{impl_downcast, DowncastSync};
use ttimer::TimeSpec;

bitflags! {
    pub struct OpenFlags: u32 {
        const O_RDONLY = 0o0;
        const O_WRONLY = 0o1;
        const O_RDWR = 0o2;
        const O_CREAT = 0o100;
        const O_EXCL = 0o200;
        const O_NOCTTY = 0o400;
        const O_TRUNC = 0o1000;
        /// The file is opened in append mode. Before each write(2), the file offset
        /// is positioned at the end of the file, as if with lseek(2). The modification
        /// of the file offset and the write operation are performed as a single atomic step.
        const O_APPEND = 0o2000;
        const O_NONBLOCK = 0o4000;
        const O_DSYNC = 0o10000;
        const O_SYNC = 0o4010000;
        const O_RSYNC = 0o4010000;
        const O_DIRECTORY = 0o200000;
        const O_NOFOLLOW = 0o400000;
        const O_CLOEXEC = 0o2000000;
        /// Enable signal-driven I/O: generate a signal (SIGIO by default, but this can
        /// be changed via fcntl(2)) when input or output becomes possible on this file
        /// descriptor. This feature is available only for terminals, pseudoterminals,
        /// sockets, and (since Linux 2.6) pipes and FIFOs.
        const O_ASYNC = 0o20000;
        const O_DIRECT = 0o40000;
        const O_LARGEFILE = 0o100000;
        const O_NOATIME = 0o1000000;
        const O_PATH = 0o10000000;
        const O_TMPFILE = 0o20200000;
    }
}

bitflags! {
    pub struct SeekWhence: u32 {
        /// set to offset bytes.
        const SEEK_SET = 0;
        /// set to its current location plus offset bytes.
        const SEEK_CUR = 1;
        /// set to the size of the file plus offset bytes.
        const SEEK_END = 2;
    }
}

bitflags! {
    pub struct StatMode: u32 {
        /// bit mask for the file type bit field
        const S_IFMT = 0o170000;
        /// socket
        const S_IFSOCK = 0o140000;
        /// symbolic link
        const S_IFLNK = 0o120000;
        /// regular file
        const S_IFREG = 0o100000;
        /// block device
        const S_IFBLK = 0o060000;
        /// directory
        const S_IFDIR = 0o040000;
        /// character device
        const S_IFCHR = 0o020000;
        /// FIFO
        const S_IFIFO = 0o010000;
        /// set-user-ID bit (see execve(2))
        const S_ISUID = 0o4000;
        /// set-group-ID bit (see below)
        const S_ISGID = 0o2000;
        /// sticky bit (see below)
        const S_ISVTX = 0o1000;
        /// owner has read, write, and execute permission
        const S_IRWXU = 0o0700;
        /// owner has read permission
        const S_IRUSR = 0o0400;
        /// owner has write permission
        const S_IWUSR = 0o0200;
        /// owner has execute permission
        const S_IXUSR = 0o0100;
        /// group has read, write, and execute permission
        const S_IRWXG = 0o0070;
        /// group has read permission
        const S_IRGRP = 0o0040;
        /// group has write permission
        const S_IWGRP = 0o0020;
        /// group has execute permission
        const S_IXGRP = 0o0010;
        /// others (not in group) have read, write,and execute permission
        const S_IRWXO = 0o0007;
        /// others have read permission
        const S_IROTH = 0o0004;
        /// others have write permission
        const S_IWOTH = 0o0002;
        /// others have execute permission
        const S_IXOTH = 0o0001;
    }
}

#[repr(C)]
#[derive(Debug)]
/// Store the file attributes from a supported file.
pub struct Stat {
    /// ID of device containing file.
    st_dev: u64,
    /// Inode number.
    st_ino: u64,
    /// File type and mode.
    st_mode: u32,
    /// Number of hard links.
    st_nlink: u32,
    /// User ID of the file's owner.
    st_uid: u32,
    /// Group ID of the file's group.
    st_gid: u32,
    /// Device ID (if special file)
    st_rdev: u64,
    __pad: u64,
    /// Size of file, in bytes.
    st_size: i64,
    /// Optimal block size for I/O.
    st_blksize: u32,
    __pad2: i32,
    /// Number 512-byte blocks allocated.
    st_blocks: u64,
    /// Backward compatibility. Used for time of last access.
    st_atime: TimeSpec,
    /// Time of last modification.
    st_mtime: TimeSpec,
    /// Time of last status change.
    st_ctime: TimeSpec,
    __unused: u64,
}

/// In UNIX, everything is a File, such as:
///
/// 1.  A normal file staying on disk.
pub trait File: DowncastSync {
    /// Reads the file starting at offset to buffer.
    ///
    /// Returns the number bytes read successfully.
    fn read_at_off(&self, off: usize, buf: &mut [u8]) -> usize;

    /// Writes the file starting at offset from buffer.
    ///
    /// Returns the number of bytes written successfully.
    fn write_at_off(&self, off: usize, buf: &[u8]) -> usize;

    /// Returns if the file is ready to read.
    fn read_ready(&self) -> bool;

    /// Returns if the file is ready to write.
    fn write_ready(&self) -> bool;
}

impl_downcast!(sync File);
