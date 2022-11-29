use alloc::sync::Arc;
use spin::{Lazy, Mutex};

mod fd;
mod link;
mod memfs;
mod stdio;

cfg_if::cfg_if! {
    if #[cfg(not(feature = "oscomp"))] {
        mod efs;
        pub use efs::FileSystem;
    } else {
        mod fat;
        pub use fat::FileSystem;
    }
}
pub use fd::*;
pub use stdio::*;

use link::get_path;
use terrno::ErrNO;
use tvfs::*;

pub static DISK_FS: Lazy<Mutex<FileSystem>> = Lazy::new(|| Mutex::new(FileSystem));

/// Opens a file object.
///
/// - `path`: Absolute path which must start with '/'.
/// - `flags`: Standard [`OpenFlags`].
/// See https://man7.org/linux/man-pages/man2/open.2.html.
///
/// 1. Check if the file exists in the [`MEM_FS`].
/// 2. Check if the file exists in the [`DISK_FS`].
pub fn open(path: &str, flags: OpenFlags) -> Result<Arc<dyn File>, ErrNO> {
    let path = Path::new(path);
    let real_path = get_path(&path);
    // TODO: Try to open file in VFS.
    let disk_file = DISK_FS.lock().open(&real_path, flags)?;
    Ok(disk_file)
}
