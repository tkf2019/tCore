//! Inode definition and operations.

use kernel_sync::SpinLock;

use crate::{SuperBlock, TimeSpec, VFSResult};

bitflags::bitflags! {
    /// Inode state bits. Protected by inode lock.
    pub struct InodeState: u8 {
        /// The inode object is modified without synchronization to disk.
        /// Not dirty enough for O_DATASYNC.
        const I_DIRTY_SYNC = 1 << 0;

        /// Data-related inode changes pending.
        const I_DIRTY_DATASYNC = 1 << 1;

        /// Data-related inode changes pending.
        const I_DIRTY_PAGESYNC = 1 << 2;

        /// The inode object is involved in an I/O transfer.
        const I_LOCK = 1 << 3;

        /// The inode object is being freed.
        const I_FREEING = 1 << 4;

        /// The inode object contents are no longer meaningful.
        const I_CLEAR = 1 << 5;

        /// The inode object has been allocated but not yet filled with data read from the disk.
        const I_NEW = 1 << 6;
    }
}

bitflags::bitflags! {
    pub struct InodeMode: u32 {
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

/// All information needed by the filesystem to handle a file is included in a data
/// structure called an inode.
#[derive(Debug)]
pub struct Inode {
    /// Inode number
    pub ino: usize,

    /// Super block
    pub sb: &'static SuperBlock,

    /// Block size in bytes
    pub block_size: usize,

    /// Inode mode
    pub mode: InodeMode,

    /// Inode inner fields
    pub inner: SpinLock<InodeInner>,
}

/// Fields of [`Inode`] protected by lock.
#[derive(Debug)]
pub struct InodeInner {
    /// File size in bytes.
    pub size: usize,

    /// Number of hard links
    pub nlink: usize,

    /// Inode state
    pub state: InodeState,

    /// Time of last access.
    pub atime: TimeSpec,

    /// Time of last modification.
    pub mtime: TimeSpec,

    /// Time of last status change.
    pub ctime: TimeSpec,
}

impl Inode {
    fn empty(sb: &SuperBlock) -> Self {
        Self {
            ino: 0,
            sb,
            block_size: 0,
            mode: InodeMode::empty(),
            inner: SpinLock::new(InodeInner {
                size: 0,
                nlink: 1,
                state: InodeState::empty(),
                atime: TimeSpec::default(),
                mtime: TimeSpec::default(),
                ctime: TimeSpec::default(),
            }),
        }
    }
}

impl Drop for Inode {
    fn drop(&mut self) {
        let inner = self.inner.lock();
        if inner.nlink == 0 {
            if let Some(delete_inode) = self.sb.ops.delete_inode {
                delete_inode(self);
            }
        }
    }
}

pub fn alloc_inode(sb: &SuperBlock) -> Inode {
    if let Some(alloc_inode) = sb.ops.alloc_inode {
        alloc_inode(sb)
    } else {
        Inode::empty(sb)
    }
}
