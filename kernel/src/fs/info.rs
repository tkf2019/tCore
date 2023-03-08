use alloc::string::String;

#[derive(Debug, Clone)]
pub struct FSInfo {
    /// The effective mode is modified by the process's umask in the usual
    /// way: in the absence of a default ACL, the mode of the created file
    /// is `(mode & ~umask)`.
    pub umask: u32,

    /// Current working directory.
    pub cwd: String,

    /// Filesystem root directory
    pub root: String,
}
