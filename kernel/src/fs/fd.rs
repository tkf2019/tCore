use alloc::{sync::Arc, vec::Vec};
use tvfs::File;

use crate::config::DEFAULT_FD_LIMIT;

/// A process-unique identifier for a file or other input/output resource,
/// such as a pipe or network socket.
pub struct FD {
    pub file: Arc<dyn File>,
}

impl FD {
    /// Creates a new [`FD`].
    pub fn new(file: Arc<dyn File>) -> Self {
        Self { file }
    }
}

/// File descriptor manager.
pub struct FDManager {
    /// List of file descriptors.
    pub list: Vec<Option<FD>>,

    /// Recycled index in the file descriptor list.
    pub recycled: Vec<usize>,

    /// Maximum file descriptor limit.
    pub limit: usize,

    /// The effective mode is modified by the process's umask in the usual
    /// way: in the absence of a default ACL, the mode of the created file
    /// is `(mode & ~umask)`.
    pub umask: u32,
}

impl FDManager {
    /// Creates a new empty [`FDManager`].
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            recycled: Vec::new(),
            limit: DEFAULT_FD_LIMIT,
            umask: 0,
        }
    }
}
