use core::cell::SyncUnsafeCell;

use vfs::{OpenFlags, Path};

use super::{FatIO, FatOCC, FatTP};

pub struct File {
    /// Open flags of the file.
    pub flags: OpenFlags,

    /// Real directory path and file name.
    pub path: Path,

    /// Real file in fat.
    pub file: SyncUnsafeCell<FatFile>,
}
