//! Inode definition and operations.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String};
use alloc::{collections::LinkedList, sync::Arc};
use kernel_sync::SpinLock;

use crate::{DentryPtr, SuperBlock};

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

pub enum InodeMode {
    /// Regular file
    Reg,

    /// Directory
    Dir,

    /// FIFO (named pipe)
    Fifo,

    /// Symbolic link
    Syn,

    /// Socket
    Sock,
}

/// All information needed by the filesystem to handle a file is included in a data
/// structure called an inode.
pub struct Inode {
    /// Inode number
    ino: usize,

    /// Block size in bytes.
    block_size: usize,

    /// File length in bytes.
    file_size: usize,

    /// Pointer to inode operation table.
    ops: Box<dyn InodeOperations>,

    /// Pointer to superblock object.
    sb: &mut SuperBlock,

    /// Inode mode
    mode: InodeMode,

    /// Inode inner members.
    inner: SpinLock<InodeMutInner>,
}

/// Inode mutable inner members.
pub struct InodeMutInner {
    /// Dentry objects referencing this inode,
    d_list: LinkedList<DentryPtr>,

    /// Number of hard links
    nlink: usize,

    /// Inode state
    state: InodeState,

    /// Time of last access.
    atime_sec: usize,
    /// Time of last access.
    atime_nsec: usize,

    /// Time of last modification.
    mtime_sec: usize,
    /// Time of last modification.
    mtime_nsec: usize,

    /// Time of last status change.
    ctime_sec: usize,
    /// Time of last status change.
    ctime_nsec: usize,
}

pub trait InodeOperations: Send + Sync {}
