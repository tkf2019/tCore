//! Dentry definition and operations.

use alloc::collections::LinkedList;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String};

use crate::{InodePtr, SuperBlock};

/// The *Common File Model* that the VFS considers each directory a file that contains a list of
/// files and other directories.
///
/// Notice that dentry objects have no corresponding image on disk, and hence no field is included
/// in the dentry structure to specify that the object has been modified.
/// 
/// Each dentry object may be in one of four states:
/// - Free The dentry object contains no valid information and is not used by the VFS. The corresponding
/// memory area is handled by the slab allocator.
/// - Unused The dentry object is not currently used by the kernel. The d_count usage counter of the object
/// is 0, but the d_inode field still points to the associated inode. The dentry object contains valid
/// information, but its contents may be discarded if necessary in order to reclaim memory.
/// - In use The dentry object is currently used by the kernel. The d_count usage counter is positive,
/// and the d_inode field points to the associated inode object. The dentry object contains valid information
/// and cannot be discarded.
/// - Negative The inode associated with the dentry does not exist, either because the corresponding disk
/// inode has been deleted or because the dentry object was created by resolving a pathname of a nonexistent
/// file. The d_inode field of the dentry object is set to NULL, but the object still remains in the dentry
/// cache, so that further lookup operations to the same file pathname can be quickly resolved. The term
/// “negative” is somewhat misleading, because no negative value is involved.
pub struct Dentry {
    /// Filename
    name: String,

    /// Pointer to dentry operation table.
    ops: Box<dyn DentryOperations>,

    /// Pointer to the superblock object of the file.
    sb: &mut SuperBlock,

    /// Dentry mutable inner members protected by RCU.
    inner: <DentryMutInner>,
}

pub struct DentryMutInner {
    /// Inode associated with filename.
    inode: Some(InodePtr),

    /// Dentry object of parent directory.
    parent: &mut Dentry,

    /// List of subdirectory dentries.
    children: LinkedList<&mut Dentry>,
}

pub trait DentryOperations: Send + Sync {}
