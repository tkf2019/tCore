use alloc::{fmt, sync::Arc, vec::Vec};
use vfs::File;

use crate::{
    config::DEFAULT_FD_LIMIT,
    error::{KernelError, KernelResult},
};

use super::{Stderr, Stdin, Stdout};

/// File descriptor manager.
#[derive(Clone)]
pub struct FDManager {
    /// List of `file descriptor`s:
    /// A process-unique identifier for a file or other input/output resource,
    /// such as a pipe or network socket.
    list: Vec<Option<Arc<dyn File>>>,

    /// Recycled index in the file descriptor list.
    recycled: Vec<usize>,

    /// Maximum file descriptor limit.
    limit: usize,

    /// The effective mode is modified by the process's umask in the usual
    /// way: in the absence of a default ACL, the mode of the created file
    /// is `(mode & ~umask)`.
    umask: u32,
}

impl FDManager {
    /// Creates a new empty [`FDManager`].
    pub fn new() -> Self {
        let mut fd_manager = Self {
            list: Vec::new(),
            recycled: Vec::new(),
            limit: DEFAULT_FD_LIMIT,
            umask: 0,
        };
        fd_manager.push(Arc::new(Stdin)).unwrap();
        fd_manager.push(Arc::new(Stdout)).unwrap();
        fd_manager.push(Arc::new(Stderr)).unwrap();
        fd_manager
    }

    /// Returns the shared reference of a [`File`].
    pub fn get(&self, fd: usize) -> KernelResult<Arc<dyn File>> {
        if fd >= self.list.len() || self.list[fd].is_none() {
            Err(KernelError::FDNotFound)
        } else {
            Ok(self.list[fd].as_ref().unwrap().clone())
        }
    }

    /// Takes the shared reference of a [`File`], leaving a [`None`] in its place.
    pub fn take(&mut self, fd: usize) -> KernelResult<Arc<dyn File>> {
        if fd >= self.list.len() || self.list[fd].is_none() {
            Err(KernelError::FDNotFound)
        } else {
            self.recycled.push(fd);
            Ok(self.list[fd].take().unwrap())
        }
    }

    /// Removes the shared reference of a [`File`].
    pub fn remove(&mut self, fd: usize) -> KernelResult {
        self.recycled.push(fd);
        self.take(fd)?;
        Ok(())
    }

    /// Allocates a new file descriptor.
    pub fn alloc(&mut self) -> KernelResult<usize> {
        if let Some(fd) = self.recycled.pop() {
            Ok(fd)
        } else {
            let fd = self.list.len();
            if fd + 1 <= self.limit {
                self.list.resize(fd + 1, None);
                Ok(fd)
            } else {
                Err(KernelError::FDOutOfBound)
            }
        }
    }

    /// Pushes a shared reference of a [`File`], resizing the list if possible.
    ///
    /// Returns the file descriptor.
    pub fn push(&mut self, file: Arc<dyn File>) -> KernelResult<usize> {
        let fd = self.alloc()?;
        self.list[fd] = Some(file);
        Ok(fd)
    }

    /// Returns the number of file descriptors.
    pub fn fd_count(&self) -> usize {
        self.list.len() - self.recycled.len()
    }

    /// Returns the limit of number.
    pub fn fd_limit(&self) -> usize {
        self.limit
    }

    /// Returns true if the number of file descriptors exceeds the limit.
    pub fn is_full(&self) -> bool {
        self.fd_count() >= self.limit
    }

    /// Returns umask.
    pub fn get_umask(&self) -> u32 {
        self.umask
    }
}

impl fmt::Debug for FDManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "File Descriptor Manager: len={:X}, limit={:X}, umask={:X}",
            self.list.len(),
            self.limit,
            self.umask
        )
    }
}
