use alloc::format;
use buddy_system_allocator::LockedFrameAllocator;
use core::{fmt, ops::Deref};
use spin::Lazy;

use super::address::{Frame, FrameRange, PhysAddr};

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

impl AllocatedFrames {
    /// Allocates frames from start to end.
    /// Use global [`FRAME_ALLOCATOR`] to track allocated frames.
    ///
    /// Throws error, otherwise allocation with the number of zero is unpredictable.
    pub fn new(count: usize) -> Option<Self> {
        assert!(
            count != 0,
            "Cannot allocate frames with the number of zero!"
        );
        if let Some(start) = FRAME_ALLOCATOR.lock().alloc(count) {
            Some(Self {
                frames: FrameRange::from_phys_addr(PhysAddr::new_canonical(start), count),
            })
        } else {
            None
        }
    }

    /// Returns the start [`Frame`] if the range is not empty.
    ///
    /// Actually, the range cannot be empty, which is guaranteed by the creation
    /// of [`AllocatedFrames`]. But the inclusive range may be exhausted by iteration.
    /// So we still need to check if the range is empty.
    pub fn start(&self) -> Option<Frame> {
        if self.is_empty() {
            None
        } else {
            Some(self.frames.start().clone())
        }
    }
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

impl Drop for AllocatedFrames {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(
            self.start()
                .expect("Nothing to deallocate. The range might have been exhausted!")
                .number(),
            self.size_in_frames(),
        );
    }
}

/// Defines global frame allocator. This implementation is based on buddy system allocator.
pub static FRAME_ALLOCATOR: Lazy<LockedFrameAllocator> = Lazy::new(|| LockedFrameAllocator::new());
