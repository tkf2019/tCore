use core::{fmt, ops::Deref};

use super::address::{Frame, FrameRange};

/// Represents a range of allocated physical memory [`Frame`]s; derefs to [`FrameRange`].
///
/// These frames are not immediately accessible because they're not yet mapped by any virtual
/// memory pages. You must do that separately in order to create a `MappedPages` type, which
/// can then be used to access the contents of these frames.
///
/// This object represents ownership of the range of allocated physical frames;
/// if this object falls out of scope, its allocated frames will be auto-deallocated upon drop.
pub struct AllocatedFrames {
    frames: FrameRange,
}

impl Deref for AllocatedFrames {
    type Target = FrameRange;
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl fmt::Debug for AllocatedFrames {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Allocated frames: {:?}", self.frames)
    }
}