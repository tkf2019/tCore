use alloc::sync::Arc;
use log::info;
use spin::{Lazy, Mutex};
use terrno::Errno;
use tvfs::*;

mod fat;
mod fd;
mod stdio;
mod pipe;
pub use fat::FileSystem;

// cfg_if::cfg_if! {
//     if #[cfg(not(feature = "oscomp"))] {
//         mod efs;
//         pub use efs::FileSystem;
//     } else {
//     }
// }
pub use fd::*;
pub use stdio::*;
pub use pipe::*;

use self::fat::FSDir;

/// Global disk filesystem.
pub static DISK_FS: Lazy<Mutex<FileSystem>> = Lazy::new(|| {
    let fs = FileSystem;

    let root = Path::root();
    fs.mkdir(&root, "dev").unwrap();
    fs.mkdir(&root, "lib").unwrap();
    fs.mkdir(&root, "tmp").unwrap();

    Mutex::new(fs)
});

/// Opens a file object.
///
/// - `path`: Absolute path which must start with '/'.
/// - `flags`: Standard [`OpenFlags`].
///
/// See https://man7.org/linux/man-pages/man2/open.2.html.
///
/// 1. Check if the file exists in the [`MEM_FS`].
/// 2. Check if the file exists in the [`DISK_FS`].
pub fn open(path: Path, flags: OpenFlags) -> Result<Arc<dyn File>, Errno> {
    /// Root is always opened.
    if path.is_root() {
        return Ok(Arc::new(FSDir::new(path)));
    }
    let mut path = path;
    let name = path.pop().unwrap();
    let pdir = get_path(&path);

    // TODO: Try to open file in VFS.

    let disk_file = DISK_FS.lock().open(&pdir, name.as_str(), flags)?;

    Ok(disk_file)
}

/// Creates a directory.
///
/// - `path`: Absolute path which must start and end with '/'.
///
/// 1. Check if parent directory is in the [`MEM_FS`].
/// 2. Try to create the directory in the [`DISK_FS`].
pub fn mkdir(path: Path) -> Result<(), Errno> {
    // Root exists.
    if path.is_root() {
        return Err(Errno::EEXIST);
    }

    // Not a directory.
    if !path.is_dir() {
        return Err(Errno::ENOTDIR);
    }

    let mut path = path;
    let name = path.pop().unwrap();
    let pdir = get_path(&path);

    // TODO: Try to create directory in VFS

    DISK_FS.lock().mkdir(&pdir, name.as_str())?;

    Ok(())
}

/// Unlinks a path.
pub fn unlink(path: Path) -> Result<(), Errno> {
    // Root cannot be unlinked.
    if path.is_root() {
        return Err(Errno::EINVAL);
    }

    if let Some(mut path) = remove_link(&path) {
        let name = path.pop().unwrap();
        DISK_FS.lock().remove(&path, name.as_str())?;
    } else {
        return Err(Errno::ENOENT);
    }

    Ok(())
}
