use alloc::{sync::Arc, vec::Vec};
use core::{fmt, ops::Deref};
use spin::Mutex;

use crate::{arch::mm::{AllocatedFrameRange, Frame}, error::{KernelError, KernelResult}};

use super::{PMArea, PMA};

/// Represents a fixed physical memory area allocated with real frames when
/// created by mapping requests from the initialization of address spaces.
///
/// Frames owned by this area will not be initialized with `0`, thus we need
/// to write the area later after creation.
pub struct FixedPMA {
    /// Allocated frames from global allocator. This area has the ownership
    /// of these frames. The physical frames will be deallocated if this fixed
    /// area is unmapped from virtual areas and dropped.
    frames: AllocatedFrameRange,
}

impl FixedPMA {
    /// Creates a new fixed area and flushes the whole area to
    /// avoid leak of private data of another task.
    pub fn new(count: usize) -> KernelResult<Self> {
        match AllocatedFrameRange::new(count, true) {
            Ok(frames) => Ok(Self { frames }),
            Err(_) => Err(KernelError::FrameAllocFailed),
        }
    }
}

impl Deref for FixedPMA {
    type Target = AllocatedFrameRange;
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl fmt::Debug for FixedPMA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pre-allocated physical memory area: [{:?}]", self.frames)
    }
}

impl PMArea for FixedPMA {
    fn is_mapped(&self) -> bool {
        true
    }

    fn get_frame(&mut self, index: usize, _: bool) -> KernelResult<Frame> {
        if index >= self.size_in_frames() {
            return Err(KernelError::FrameOutOfRange);
        }
        Ok(Frame::from(self.frames.frames.start + index))
    }

    fn get_frames(&mut self, _: bool) -> KernelResult<Vec<Option<Frame>>> {
        Ok(self.range().map(|frame| Some(frame)).collect())
    }

    fn check_frame(&self, index: usize) -> bool {
        index >= self.size_in_frames()
    }

    fn split(
        &mut self,
        start: Option<usize>,
        end: Option<usize>,
    ) -> KernelResult<(Option<PMA>, Option<PMA>)> {
        if start.is_none() && end.is_none() {
            Ok((None, None))
        } else if end.is_none() {
            let start = self.frames.start + start.unwrap();

            if start >= self.frames.end {
                return Err(KernelError::PMAOutOfRange);
            }

            let right = self.frames.split_at(start, false).unwrap();
            Ok((Some(Arc::new(Mutex::new(Self { frames: right }))), None))
        } else if start.is_none() {
            let end = self.frames.start + end.unwrap();

            if end >= self.frames.end {
                return Err(KernelError::PMAOutOfRange);
            }

            let left = self.frames.split_at(end, true).unwrap();
            Ok((Some(Arc::new(Mutex::new(Self { frames: left }))), None))
        } else {
            let start = self.frames.start + start.unwrap();
            let end = self.frames.end + end.unwrap();

            if start == end {
                return Ok((None, None));
            }

            if start > end {
                return Err(KernelError::InvalidArgs);
            }

            if start >= self.frames.end || end >= self.frames.end {
                return Err(KernelError::PMAOutOfRange);
            }

            let right = self.frames.split_at(end, false).unwrap();
            let right = Arc::new(Mutex::new(Self { frames: right }));
            let mid = self.frames.split_at(start, false).unwrap();
            let mid = Arc::new(Mutex::new(Self { frames: mid }));
            Ok((Some(mid), Some(right)))
        }
    }
}
