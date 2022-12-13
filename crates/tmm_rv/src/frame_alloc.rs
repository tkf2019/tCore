use buddy_system_allocator::LockedFrameAllocator;
use core::{fmt, ops::Deref};
use log::{info, trace};
use spin::Lazy;

use crate::{Frame, FrameRange, PageTable, PAGE_SIZE};

/// Defines global frame allocator. This implementation is based on buddy system allocator.
pub static GLOBAL_FRAME_ALLOCATOR: Lazy<LockedFrameAllocator> =
    Lazy::new(|| LockedFrameAllocator::new());

/// Global interface for frame allocator.
pub fn frame_alloc(count: usize) -> Option<usize> {
    GLOBAL_FRAME_ALLOCATOR.lock().alloc(count)
}

/// Global interface for frame deallocator
pub fn frame_dealloc(start: usize, count: usize) {
    GLOBAL_FRAME_ALLOCATOR.lock().dealloc(start, count)
}

/// Initialize global frame allocator
pub fn frame_init(start: usize, end: usize) {
    info!("Global Frame Allocator [{:#x}, {:#x})", start, end);
    GLOBAL_FRAME_ALLOCATOR.lock().add_frame(start, end)
}

/// A wrapper of allocated physical memory [`Frame`].
///
/// The frame is not immediately accessible because they're not yet mapped by any virtual
/// memory page. You must do that separately in order to create a [`AllocatedPages`] or
/// [`PageTable`] type, which can then be used to access the contents of these frames.
///
/// This object represents ownership of the allocated physical frame.
/// If this object falls out of scope, this frame will be auto-deallocated upon drop.
pub struct AllocatedFrame {
    frame: Frame,
}

impl AllocatedFrame {
    /// Allocates a single frame.
    /// Use global [`GLOBAL_FRAME_ALLOCATOR`] to track allocated frames.
    pub fn new(flush: bool) -> Result<Self, &'static str> {
        if let Some(frame) = frame_alloc(1) {
            let frame = Frame::from(frame);
            if flush {
                unsafe {
                    core::ptr::write_bytes(frame.start_address().value() as *mut u8, 0, PAGE_SIZE)
                };
            }
            // trace!("AllocatedFrame {:?}", frame);
            Ok(Self { frame })
        } else {
            Err("Failed to allocate frame.")
        }
    }
}

impl Deref for AllocatedFrame {
    type Target = Frame;
    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl fmt::Debug for AllocatedFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Allocated {:?}", self.frame)
    }
}

impl Drop for AllocatedFrame {
    fn drop(&mut self) {
        frame_dealloc(self.number(), 1);
    }
}

/// Represents a range of allocated physical memory [`Frame`]s; derefs to [`FrameRange`].
///
/// These frames are not immediately accessible because they're not yet mapped by any virtual
/// memory pages. You must do that separately in order to create a [`AllocatedFrameRange`] or
/// [`PageTable`] type, which can then be used to access the contents of these frames.
///
/// This object represents ownership of the range of allocated physical frames;
/// if this object falls out of scope, its allocated frames will be auto-deallocated upon drop.
pub struct AllocatedFrameRange {
    pub frames: FrameRange,
}

impl AllocatedFrameRange {
    /// Allocates frames from start to end.
    /// Use global [`GLOBAL_FRAME_ALLOCATOR`] to track allocated frames.
    ///
    /// # Argument
    ///
    /// - `count`: the number of frames
    /// - `flush`: if set, flush the memory region with the start address of allocated frames.
    ///
    /// # Return
    ///
    /// Throws error, otherwise allocation with the number of zero is unpredictable.
    pub fn new(count: usize, flush: bool) -> Result<Self, &'static str> {
        if count == 0 {
            return Err("Cannot allocate frames with zero count.");
        }
        if let Some(start) = frame_alloc(count) {
            let start = Frame::from(start);
            let end = Frame::from(start + count);
            if flush {
                unsafe {
                    core::ptr::write_bytes(
                        start.start_address().value() as *mut u8,
                        0,
                        PAGE_SIZE * count,
                    )
                };
            }
            // trace!("AllocatedFrames {:?}", FrameRange::new(start, end));
            Ok(Self {
                frames: FrameRange::new(start, end),
            })
        } else {
            Err("Failed to allocate frame.")
        }
    }

    /// Splits this [`AllocatedFrameRange`] into two separate objects:
    /// - `[beginning : at_frame - 1]`
    /// - `[at_frame : end]`
    ///
    /// Returns [`None`] if `at_frame` is otherwise out of bounds.
    pub fn split_at(&mut self, at_frame: Frame, new_below: bool) -> Option<Self> {
        let (left, right) = if at_frame == self.start {
            (FrameRange::empty(), FrameRange::new(at_frame, self.start))
        } else if at_frame == self.end {
            (FrameRange::new(self.start, at_frame), FrameRange::empty())
        } else if at_frame > self.start && at_frame < self.end {
            (
                FrameRange::new(self.start, at_frame),
                FrameRange::new(at_frame, self.end),
            )
        } else {
            return None;
        };
        if new_below {
            self.frames = right;
            Some(Self { frames: left })
        } else {
            self.frames = left;
            Some(Self { frames: right })
        }
    }
}

impl Deref for AllocatedFrameRange {
    type Target = FrameRange;
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl fmt::Debug for AllocatedFrameRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Allocated {:?}", self.frames)
    }
}

impl Drop for AllocatedFrameRange {
    fn drop(&mut self) {
        frame_dealloc(self.start.number(), self.size_in_frames());
    }
}
