use alloc::{collections::BTreeMap, rc::Rc, vec::Vec};
use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use log::warn;
use tmm_rv::{AllocatedFrames, Frame, PhysAddr};

use crate::error::{KernelError::FrameOutOfRange, KernelResult};

pub trait PMArea: Debug + Send + Sync {
    /// Gets target frame by index.
    ///
    /// Returns [`FrameOutOfRange`] if the `index` is out of the range of
    /// allocated frames.
    fn get_frame(&self, index: usize) -> KernelResult<Frame>;
}

/// Represents a fixed physical memory area allocated with real frames when
/// created by mapping requests from the initialization of address spaces.
pub struct FixedPMA {
    /// Allocated frames from global allocator. This area has the ownership
    /// of these frames. The physical frames will be deallocated if this fixed
    /// area is unmapped from virtual areas and dropped.
    frames: AllocatedFrames,
}

impl FixedPMA {
    fn new(count: usize) -> KernelResult<Self> {
        match AllocatedFrames::new(count) {
            Ok(frames) => Ok(Self { frames }),
            Err(err) => {
                warn!("{}", err);
                Err(KernelError::FrameAllocFailed)
            }
        }
    }
}

impl Deref for FixedPMA {
    type Target = AllocatedFrames;
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl PMArea for FixedPMA {
    fn get_frame(&self, index: usize) -> KernelResult<Frame> {
        if index >= self.size_in_frames() {
            return Err(FrameOutOfRange);
        }
        Ok(Frame::from(self.start() + index))
    }
}
