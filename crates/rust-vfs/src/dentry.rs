//! Dentry definition and operations.

use core::cell::SyncUnsafeCell;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;
use alloc::{
    collections::LinkedList,
    sync::{Arc, Weak},
};
use kernel_sync::SeqLock;

use crate::{Inode, InodeState, VFS};

/// The **Common File Model** that the VFS considers each directory a file that contains a list of
/// files and other directories.
///
/// Notice that dentry objects have no corresponding image on disk, and hence no field is included
/// in the dentry structure to specify that the object has been modified.
///
/// Each dentry object may be in one of four states:
/// - **Free**: The dentry object contains no valid information and is not used by the VFS.
/// The corresponding memory area is handled by the slab allocator.
/// - **Unused**: The dentry object is not currently used by the kernel. The reference counter of the
/// object is 0, but the `inode` field still points to the associated inode. The dentry object contains valid
/// information, but its contents may be discarded if necessary in order to reclaim memory.
/// - **In use**: The dentry object is currently used by the kernel. The 'inode' field points to the associated
/// inode object. The dentry object contains valid information and cannot be discarded.
/// - **Negative**: The inode associated with the dentry does not exist, either because the corresponding disk
/// inode has been deleted or because the dentry object was created by resolving a pathname of a nonexistent
/// file. The `inode` field of the dentry object is set to `None`, but the object still remains in the dentry
/// cache, so that further lookup operations to the same file pathname can be quickly resolved. The term
/// "negative" is somewhat misleading, because no negative value is involved.
pub struct Dentry {
    /// Filename
    pub name: String,

    /// Inode associated with filename.
    pub inode: SyncUnsafeCell<Option<Weak<Inode>>>,

    /// Dentry of parent directory.
    pub parent: Weak<Dentry>,

    /// List of subdirectory dentries.
    pub children: SeqLock<LinkedList<Arc<Dentry>>>,
}

impl Dentry {
    /// Creates a new [`Dentry`] in **negative** state.
    pub fn new(name: &str, parent: Weak<Dentry>) -> Self {
        Self {
            name: String::from(name),
            inode: SyncUnsafeCell::new(None),
            parent,
            children: SeqLock::new(LinkedList::new()),
        }
    }

    /// Searches a directory for an inode corresponding to the filename included in a dentry object.
    pub fn find(&self, name: &str) -> Option<Arc<Dentry>> {
        self.children.read(|list| {
            for dentry in list {
                if dentry.name == name {
                    return Some(dentry.clone());
                }
            }
            None
        })
    }

    /// Creates a new dentry in this directory, otherwise returns the existing dentry.
    pub fn create(&self, name: &str, this: Weak<Dentry>) -> Arc<Dentry> {
        let mut locked = self.children.write();
        for dentry in locked.iter() {
            if dentry.name == name {
                return dentry.clone();
            }
        }
        let new_dentry = Arc::new(Dentry::new(name, this));
        locked.push_front(new_dentry.clone());
        new_dentry
    }

    /// Removes a dentry in this directory.
    pub fn remove(&self, name: &str) {
        let mut locked = self.children.write();
        locked.drain_filter(|dentry| dentry.name == name);
    }

    /// Sets the inode pointer of this [`Dentry`].
    pub fn set_inode(&self, inode: Weak<Inode>) {
        *unsafe { &mut *self.inode.get() } = Some(inode);
    }

    /// Gets the inode pinter of this [`Dentry`].
    pub fn get_inode(&self) -> Option<Arc<Inode>> {
        unsafe { &*self.inode.get() }.as_ref().and_then(|inode| {
            let inode = inode.upgrade().unwrap();
            if inode.locked.read(|locked| locked.state == InodeState::Clear) {
                return None;
            }
            Some(inode)
        })
    }
}

/// Maximum Dcache size.
pub const DCACHE_SIZE: usize = 512;

/// LRU Dcache for memory reclamation.
pub struct DentryCache(LinkedList<Arc<Dentry>>);

impl DentryCache {
    /// Creates a new [`DentryCache`]
    pub const fn new() -> Self {
        Self(LinkedList::new())
    }

    /// Pushes the dentry recently accessed to the front.
    ///
    /// We don't need to consider duplicated dentries, because these nodes will be released sooner or later
    /// and reference counter will be decreased.
    ///
    /// This function might acquire the lock of parent dentry.
    pub fn access(&mut self, d: Arc<Dentry>) {
        self.0.push_front(d);

        if self.0.len() >= DCACHE_SIZE {
            self.reclaim();
        }
    }

    /// Reclaims the node which has not been accessed most recently.
    ///
    /// The node can be categorized as following (a node might satisfy multiple charateritics below):
    /// 1. Leaf node that has no child
    /// 2. Invalid dentry that points to inode
    fn reclaim(&mut self) {
        if let Some(last) = self.0.pop_back() {
            if let Some(parent) = last.parent.upgrade() {
                // acquire the lock here
                let mut locked = parent.children.write();
                if Arc::strong_count(&last) == 2 {
                    locked.drain_filter(|entry| entry.name == last.name);
                }
            }
        }
    }
}
