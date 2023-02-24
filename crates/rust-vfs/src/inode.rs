pub enum InodeState {
    /// Newly created but not synchronized to disk yet. 
    New,

    /// Inode modified but not synchronized to disk yet.
    Dirty,
    Invalid,
    Locked,
    Freed,
}

pub struct Inode {}