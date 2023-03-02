//! `SuperBlock` definition and operation.

use core::marker::PhantomData;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use kernel_sync::SpinLock;

use crate::{Dentry, DentryCache, InodeCache, InodeMode, InodeState, Path, VFSResult, VFS};

bitflags::bitflags! {
    pub struct MountFlags: u64 {}
}

/// A `SuperBlock` stores information concerning a mounted filesystem. For disk-based filesystems,
/// this object usually corresponds to a filesystem control block stored on disk.
pub struct SuperBlock<V: VFS> {
    /// Block size in bytes.
    pub block_size: usize,

    /// Maximum size of files in bytes.
    pub max_size: usize,

    /// Filesystem name.
    pub name: String,

    /// Mount flags
    pub flags: MountFlags,

    /// Dentry object of the filesystem’s root directory.
    ///
    /// [`SuperBlock`] holds a reference to avoid root being dropped.
    pub root: Arc<Dentry<V>>,

    /// Inner mutable members
    pub inner: SpinLock<SuperBlockMutInner<V>>,

    /// Dentry cache
    pub dcache: SpinLock<DentryCache<V>>,

    /// Inode cache
    pub icache: SpinLock<InodeCache<V>>,
}

pub struct SuperBlockMutInner<V: VFS> {
    phantom: PhantomData<V>,

    /// Modified (dirty) flag, which specifies whether the superblock is dirty.
    pub dirty: bool,
}

impl<V: VFS> SuperBlock<V> {
    pub fn mount(name: &str, block_size: usize, max_size: usize, flags: MountFlags) -> Self {
        Self {
            block_size,
            max_size,
            name: String::from(name),
            flags,
            root: Arc::new(Dentry::new("/", Weak::new())),
            inner: SpinLock::new(SuperBlockMutInner {
                phantom: PhantomData,
                dirty: false,
            }),
            dcache: SpinLock::new(DentryCache::new()),
            icache: SpinLock::new(InodeCache::new()),
        }
    }
}

impl<V: VFS> SuperBlock<V> {
    /// Finds a dentry with the given absuolute path.
    ///
    /// Returns the last parent directory cached through the path.
    fn find_dentry(&self, path: &Path) -> Result<Arc<Dentry<V>>, (Arc<Dentry<V>>, usize)> {
        let mut curr = self.root.clone();
        for (i, name) in path.split().into_iter().enumerate() {
            let next = curr.children.read(|list| {
                for dentry in list {
                    if dentry.name == name {
                        return Some(dentry.clone());
                    }
                }
                None
            });
            if next.is_none() {
                return Err((curr.clone(), i));
            } else {
                curr = next.unwrap();
                self.dcache.lock().access(curr.clone());
            }
        }
        return Ok(curr);
    }

    /// Searches a directory for an dentry corresponding to the name.
    pub fn lookup(&self, dir: &Path, name: &str) -> VFSResult<Arc<Dentry<V>>> {
        // Searches in the parent directory
        let mut path = dir.clone();
        path.extend(name);
        self.find_dentry(&path)
            .or_else(|(pdir, rest)| {
                // Creates dentries through the directory path, leaving branch dentries **negative**.
                let mut curr = pdir;
                for name in dir.split()[rest..].into_iter() {
                    curr = curr.create(*name, Arc::downgrade(&curr));
                    self.dcache.lock().access(curr.clone());
                }
                Ok(curr.clone())
            })
            .and_then(|dentry| {
                if dentry.get_inode().is_none() {
                    // Lookup inode on disk
                    let inode = V::lookup(dir, name)?;
                    self.icache.lock().access(inode.clone());
                    dentry.set_inode(Arc::downgrade(&inode));
                }
                Ok(dentry)
            })
    }

    /// Creates a new disk inode for a file associated with a dentry object in some directory.
    pub fn create(&self, dir: &mut Path, name: &str, mode: InodeMode, is_dir: bool) -> VFSResult<Arc<Dentry<V>>> {
        let pdir = if let Some(pdir) = dir.pop() {
            self.lookup(dir, &pdir)?
        } else {
            self.root.clone()
        };
        let dentry = pdir.create(name, Arc::downgrade(&pdir));
        if dentry.get_inode().is_none() {
            if is_dir {
                dentry.set_inode(Arc::downgrade(&V::mkdir(&pdir, name, mode)?));
            } else {
                dentry.set_inode(Arc::downgrade(&V::create(&pdir, name, mode)?));
            }
        }
        Ok(dentry)
    }

    /// Removes the hard link of the file specified by a dentry object from directory by step:
    ///
    /// 1. Unlinks the inode on disk and removes the inode if necessary.
    /// 2. Removes the dentry on disk if possible.
    /// 3. Decreases the hard link of [`Inode`].
    /// 4. Removes the [`Dentry`] from memory.
    pub fn unlinkat(&self, dir: &Path, name: &str) -> VFSResult {
        self.lookup(dir, name).and_then(|dentry| {
            // Unlink the inode
            let inode = dentry.get_inode().unwrap();
            // Read parent inode from disk
            let pdir = dentry.parent.upgrade().unwrap();
            if pdir.get_inode().is_none() {
                // Lookup inode on disk
                let mut path = dir.clone();
                let pdir_name = path.pop().unwrap();
                let inode = V::lookup(&path, &pdir_name)?;
                self.icache.lock().access(inode.clone());
                pdir.set_inode(Arc::downgrade(&inode));
            }

            V::unlink(&pdir, &dentry)?;

            inode.unlink()?;

            // We can call `unwrap()` because `lookup` returns something
            dentry.parent.upgrade().unwrap().remove(name);

            Ok(())
        })
    }
}
