//! A virtual filesystem library implemented in Rust.

#![crate_type = "lib"]
#![crate_name = "vfs"]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(sync_unsafe_cell)]
#![feature(drain_filter)]

extern crate log;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

mod dentry;
mod file;
mod inode;
mod path;
mod superblock;

pub use dentry::*;
pub use file::*;
pub use inode::*;
pub use path::*;
pub use superblock::*;

use alloc::sync::Arc;
use errno::Errno;

pub type VFSResult<T = ()> = Result<T, Errno>;

pub trait VFS: Sized {
    /// Allocates space for an inode object, including the space required for filesystem specific data.
    fn alloc_inode() -> VFSResult<Arc<Inode<Self>>>;

    /// Destorys an inode object, including the filesystem-specific data;
    /// the `ino` field of the inode object identifies the specific filesystem inode on the disk to be destroyed.
    fn destroy_inode(inode: &Inode<Self>) -> VFSResult;

    /// Fills the fields of the inode object with the data on disk;
    /// the `ino` field of the inode object identifies the specific filesystem inode on the disk to be read.
    fn read_inode(inode: &Inode<Self>) -> VFSResult;

    /// Updates a filesystem inode with the contents of the inode object passed as the parameter;
    /// the `ino` field of the inode object identifies the filesystem inode on disk that is concerned.
    fn flush_inode(inode: &Inode<Self>) -> VFSResult;

    /// Searches a directory for an inode corresponding to the filename included in a dentry object.
    fn lookup(dir: &Path, name: &str) -> VFSResult<Arc<Inode<Self>>>;

    /// Creates a new disk inode for a file associated with a dentry object in some directory.
    fn create(pdir: &Dentry<Self>, name: &str, mode: InodeMode) -> VFSResult<Arc<Inode<Self>>>;

    /// Creates a new hard link that refers to the file specified by old_dentry in the directory
    /// dir; the new hard link has the name specified by new_dentry.
    fn link(old: &Dentry<Self>, dir: &Path, new: &Dentry<Self>) -> VFSResult;

    /// Removes the hard link of the file specified by an inode object from a directory.
    fn unlink(pdir: &Dentry<Self>, dentry: &Dentry<Self>) -> VFSResult;

    /// Creates a new inode for a symbolic link associated with a dentry object in some directory.
    fn symlink(dir: &Path, dentry: &Dentry<Self>, name: &str) -> VFSResult<Arc<Inode<Self>>>;

    /// Creates a new inode for a directory associated with a dentry object in some directory.
    fn mkdir(pdir: &Dentry<Self>, name: &str, mode: InodeMode) -> VFSResult<Arc<Inode<Self>>>;

    /// Removes from a directory the subdirectory whose name is included in a dentry object.
    fn rmdir(dir: &Path, dentry: &Dentry<Self>) -> VFSResult;

    /// Moves the file identified by old_entry from the old_dir directory to the new_dir one.
    /// The new filename is included in the dentry object that new_dentry points to.
    fn rename(old_dir: &Path, old_dentry: &Dentry<Self>, new: &Path, new_dentry: &Dentry<Self>) -> VFSResult;

    /// Invoked when flushing the filesystem to update filesystem-specific data structures on disk
    /// (used by journaling filesystems)
    fn sync() -> VFSResult {
        Ok(())
    }
}
