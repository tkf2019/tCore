//! `SuperBlock` definition and operation.

use alloc::{collections::BTreeMap, sync::Arc};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{string::String, vec::Vec};
use kernel_sync::{Mutex, MutexGuard};

use crate::{File, FileSystemType, Inode};

/// Information of block device.
pub struct BlockDeviceInfo {
    name: String,
}

/// A `SuperBlock` stores information concerning a mounted filesystem. For disk-based filesystems,
/// this object usually corresponds to a filesystem control block stored on disk.
pub struct SuperBlock {
    /// Block size in bytes.
    block_size: usize,

    /// Maximum size of files in bytes.
    max_size: usize,

    /// Modified (dirty) flag.
    dirty: bool,

    /// Operations wrapped by mutex lock.
    op: Mutex<dyn SuperOperations>,

    /// Filesystem type.
    fs_type: &'a mut FileSystemType,

    /// All inodes loaded or generated from disk.
    ///
    /// These inodes are mapped with the inode number.
    inodes: BTreeMap<usize, Arc<Mutex<Inode>>>,

    /// List of modified inodes.
    dirty_inodes: Vec<usize>,

    /// List of inodes waiting to be written to disk.
    io_inodes: Vec<usize>,

    /// Mount flags
    flags: MountFlags,

    /// Dentry object of the filesystem’s root directory.
    ///
    /// `SuperBlock` holds a reference to avoid root being dropped.
    root: Arc<Mutex<Dentry>>,

    /// List of file objects.
    files: Vec<Arc<Mutex<File>>>,
}

pub trait SuperOperations: Send + Sync {
    /// Allocates space for an inode object, including the space required for filesystemspecific data.
    fn alloc_inode(sb: &mut MutexGuard<SuperBlock>) -> Arc<Mutex<Inode>>;
}
