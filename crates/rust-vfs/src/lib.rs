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
mod superblock;

pub use dentry::*;
pub use file::*;
pub use fs::*;
pub use inode::*;
pub use superblock::*;
