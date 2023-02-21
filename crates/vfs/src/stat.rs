use bitflags::bitflags;

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

/// Store the file attributes from a supported file.
#[repr(C)]
#[derive(Debug, Default)]
pub struct Stat {
    /// ID of device containing file.
    pub st_dev: u64,
    /// Inode number.
    pub st_ino: u64,
    /// File type and mode.
    pub st_mode: u32,
    /// Number of hard links.
    pub st_nlink: u32,
    /// User ID of the file's owner.
    pub st_uid: u32,
    /// Group ID of the file's group.
    pub st_gid: u32,
    /// Device ID (if special file)
    pub st_rdev: u64,
    __pad: u64,
    /// Size of file, in bytes.
    pub st_size: u64,
    /// Optimal block size for I/O.
    pub st_blksize: u32,
    __pad2: u32,
    /// Number 512-byte blocks allocated.
    pub st_blocks: u64,
    /// Time of last access.
    pub st_atime_sec: usize,
    /// Time of last access.
    pub st_atime_nsec: usize,
    /// Time of last modification.
    pub st_mtime_sec: usize,
    /// Time of last modification.
    pub st_mtime_nsec: usize,
    /// Time of last status change.
    pub st_ctime_sec: usize,
    /// Time of last status change.
    pub st_ctime_nsec: usize,
    __unused: u64,
}
