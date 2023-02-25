//! A virtual filesystem library implemented in Rust.

#![crate_type = "lib"]
#![crate_name = "vfs"]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate log;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

mod dentry;
mod file;
mod fs;
mod inode;
mod path;
mod superblock;

pub use dentry::*;
pub use file::*;
pub use fs::*;
pub use inode::*;
pub use path::*;
pub use superblock::*;

use alloc::sync::Arc;
use kernel_sync::SpinLock;

/// Pointer to [`Inode`] wrapped with spin SpinLock and reference counter.
pub type InodePtr = Arc<Inode>;

/// Pointer to [`File`] wrapped with spin SpinLock and reference counter.
pub type FilePtr = Arc<File>;

/// Pointer to [`Dentry`] wrapped with spin SpinLock and reference counter.
pub type DentryPtr = Arc<Dentry>;
