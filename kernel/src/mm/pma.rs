use alloc::vec::Vec;
use core::{fmt, ops::Deref};
use log::{debug, info, warn};
use tmm_rv::{AllocatedFrames, Frame};

use crate::error::{KernelError, KernelResult};

pub trait PMArea: fmt::Debug + Send + Sync {
    /// Returns true if this area is mapped.
    fn is_mapped(&self) -> bool;

    /// Gets target frame by index.
    ///
    /// Returns [`FrameOutOfRange`] if the `index` is out of the range of
    /// allocated frames.
    fn get_frame(&self, index: usize) -> KernelResult<Frame>;

    /// Get serialized frames in the order of allocation.
    fn get_frames(&self) -> Vec<Frame>;
}

/// Represents an virtual memory area identically mapped, usually kernel
/// address space sections.
pub struct IdenticalPMA;

impl fmt::Debug for IdenticalPMA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Indentically mapped physical memory area")
    }
}

impl PMArea for IdenticalPMA {
    fn is_mapped(&self) -> bool {
        false
    }

    fn get_frame(&self, _: usize) -> KernelResult<Frame> {
        unimplemented!("Indentical physical memory area cannot be referenced as frames.")
    }
    fn get_frames(&self) -> Vec<Frame> {
        unimplemented!("Indentical physical memory area cannot be referenced as frames.")
    }
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
    pub fn new(count: usize) -> KernelResult<Self> {
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

impl fmt::Debug for FixedPMA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fixed [{:?}]", self.frames)
    }
}

impl PMArea for FixedPMA {
    fn is_mapped(&self) -> bool {
        true
    }

    fn get_frame(&self, index: usize) -> KernelResult<Frame> {
        if index >= self.size_in_frames() {
            return Err(KernelError::FrameOutOfRange);
        }
        Ok(Frame::from(self.start + index))
    }

    fn get_frames(&self) -> Vec<Frame> {
        self.range().collect()
    }
}
