//! `SuperBlock` definition and operation.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String, vec::Vec};
use alloc::{
    collections::{BTreeMap, LinkedList},
    sync::Arc,
};
use core::{cell::SyncUnsafeCell, marker::PhantomData, ptr::NonNull};
use kernel_sync::{SpinLock, SpinLockGuard};
use spin::Lazy;

use crate::{DentryPtr, File, FilePtr, FileSystemType, Inode, InodePtr};

bitflags::bitflags! {
    pub struct MountFlags: u64 {}
}

/// A `SuperBlock` stores information concerning a mounted filesystem. For disk-based filesystems,
/// this object usually corresponds to a filesystem control block stored on disk.
pub struct SuperBlock {
    /// Block size in bytes.
    block_size: usize,

    /// Maximum size of files in bytes.
    max_size: usize,

    /// Filesystem type.
    ///
    /// A raw mutable reference, for [`FileSystemType`] must live longer.
    fs_type: &mut FileSystemType,

    /// Mount flags
    flags: MountFlags,

    /// Dentry object of the filesystem’s root directory.
    ///
    /// `SuperBlock` holds a reference to avoid root being dropped.
    root: DentryPtr,

    /// Pointer to superblock information of a specific filesystem.
    ///
    /// VFS allows filesystems to act directly on the `fs_info` field of [`SuperBlock`] in memory without
    /// accessing the disk.
    fs_info: usize,

    /// Pointer to super block operations table.
    ops: Box<dyn SuperOperations>,

    /// Inner mutable members
    inner: SpinLock<SuperBlockMutInner>,
}

pub trait SuperOperations: Send + Sync {}

pub struct SuperBlockMutInner {
    /// Modified (dirty) flag, which specifies whether the superblock is dirty.
    dirty: bool,

    /// All inodes loaded or generated from disk.
    ///
    /// These inodes are mapped with the inode number.
    inodes: BTreeMap<usize, InodePtr>,

    /// The list of valid unused inodes, typically those mirroring valid disk inodes and not
    /// currently used by any process.
    unused_inodes: LinkedList<InodePtr>,

    /// The list of in-use inodes, that is, those mirroring valid disk inodes and used by some process.
    inuse_inodes: LinkedList<InodePtr>,

    /// List of modified inodes.
    dirty_inodes: LinkedList<InodePtr>,

    /// List of inodes waiting to be written to disk.
    io_inodes: LinkedList<InodePtr>,

    /// List of file objects.
    files: LinkedList<FilePtr>,
}

impl SuperBlock {
    pub const fn mount<O: SuperOperations>(
        fs_type: &mut FileSystemType,
        block_size: usize,
        max_size: usize,
        flags: MountFlags,
        root: DentryPtr,
        ops: O,
    ) -> Self {
        Self {
            block_size,
            max_size,
            fs_type,
            flags,
            root,
            fs_info: 0,
            ops: Box::new(O),
            inner: SpinLock::new(SuperBlockMutInner {
                dirty: false,
                inodes: BTreeMap::new(),
                unused_inodes: LinkedList::new(),
                inuse_inodes: LinkedList::new(),
                dirty_inodes: LinkedList::new(),
                io_inodes: LinkedList::new(),
                files: LinkedList::new(),
            }),
        }
    }
}
