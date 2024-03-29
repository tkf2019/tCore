//!An easy file system isolated from the kernel
#![no_std]
// #![deny(missing_docs)]
extern crate alloc;
mod bitmap;
mod block_cache;
mod efs;
mod file;
mod layout;
mod vfs;
/// Use a block size of 512 bytes
pub const BLOCK_SZ: usize = 512;
use bitmap::Bitmap;
use block_cache::{block_cache_sync_all, get_block_cache};
pub use efs::EasyFileSystem;
pub use file::*;
use layout::*;
pub use device_cache::BlockDevice;
pub use vfs::Inode;
