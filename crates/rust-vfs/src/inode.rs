//! Inode definition and operations.

use core::{cell::SyncUnsafeCell, marker::PhantomData};

use alloc::{collections::LinkedList, sync::Arc};
use kernel_sync::SeqLock;

use crate::{TimeSpec, VFSResult, VFS};

/// Inode state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeState {
    /// Modified without synchronization to disk.
    Dirty,

    /// The inode object is involved in an I/O transfer.
    Lock,

    /// The inode object is being freed.
    Freeing,

    /// The inode object contents are no longer meaningful.
    Clear,

    /// The inode object has been allocated but not yet filled with data read from the disk.
    New,
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
pub struct Inode<V: VFS> {
    pub phantom: PhantomData<V>,

    /// Inode number
    pub ino: usize,

    /// Block size in bytes
    pub block_size: usize,

    /// Inode mutable members
    pub inner: SyncUnsafeCell<InodeMutInner>,

    /// Inode locked members
    pub locked: SeqLock<InodeLockedInner>,
}

/// Inode mutable inner members initialized after allocating an [`Inode`].
pub struct InodeMutInner {
    /// Inode mode
    pub mode: InodeMode,
}

/// Inode locked inner members.
pub struct InodeLockedInner {
    /// File length in bytes.
    pub file_size: usize,

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

impl<V: VFS> Inode<V> {
    /// Decreases the hard link of this inode, destroying the disk inode if the hard link is zero.
    pub fn unlink(&self) -> VFSResult {
        let mut locked = self.locked.write();
        if locked.nlink == 1 {
            locked.state = InodeState::Clear;
            V::destroy_inode(self)?;
        }
        locked.nlink -= 1;
        Ok(())
    }

    /// Gets [`InodeMode`] of this [`Inode`].
    pub fn get_mode(&self) -> InodeMode {
        unsafe { &*self.inner.get() }.mode
    }

    /// Sets [`InodeMode`] of this [`Inode`].
    pub fn set_mode(&self, mode: InodeMode) {
        unsafe { &mut *self.inner.get() }.mode = mode;
    }
}

impl<V: VFS> Drop for Inode<V> {
    fn drop(&mut self) {
        self.locked.read(|locked| {
            if locked.state == InodeState::Dirty {
                V::flush_inode(self).unwrap();
            }
        })
    }
}

/// Maximum Icache size.
pub const ICACHE_SIZE: usize = 512;

/// LRU Icache for memory reclamation
pub struct InodeCache<V: VFS>(LinkedList<Arc<Inode<V>>>);

impl<V: VFS> InodeCache<V> {
    /// Creates a new [`InodeCache`].
    pub const fn new() -> Self {
        Self(LinkedList::new())
    }

    /// Pushes the node recently accessed to the front.
    ///
    /// We don't need to consider duplicated nodes, because these nodes will be released sooner or later
    /// and reference counter will be decreased.
    pub fn access(&mut self, inode: Arc<Inode<V>>) {
        self.0.push_front(inode);

        if self.0.len() >= ICACHE_SIZE {
            self.0.pop_back();
        }
    }
}
