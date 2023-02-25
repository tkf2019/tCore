//! A naive seqlock implementation.
//!
//! Seqlocks are similar to read/write spin locks, except they give a much higher
//! priority to writers: in fact a writer is allowed to proceed even when readers
//! are active.

use crate::SpinLock;

pub struct SeqLock<T: ?Sized> {
    seq: usize,
    lock: SpinLock<T>,
}

// TODO
