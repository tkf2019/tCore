mod fixed;
mod identical;
mod lazy;

use alloc::{sync::Arc, vec::Vec};
use core::fmt;
use spin::Mutex;

use crate::{
    arch::mm::Frame,
    error::{KernelError, KernelResult},
};

use super::file::BackendFile;
pub use fixed::*;
pub use identical::*;
pub use lazy::*;

pub type PMA = Arc<Mutex<dyn PMArea>>;

pub trait PMArea: fmt::Debug + Send + Sync {
    /// Returns true if this area is mapped.
    fn is_mapped(&self) -> bool {
        false
    }

    /// Gets target frame by index.
    ///
    /// # Error
    /// - [`KernelError::FrameOutOfRange`]: if the `index` is out of the range of
    /// allocated frames.
    fn get_frame(&mut self, _index: usize, _alloc: bool) -> KernelResult<Frame> {
        Err(KernelError::Unimplemented)
    }

    /// Get all frames in this area in order of allocation.
    fn get_frames(&mut self, _alloc: bool) -> KernelResult<Vec<Option<Frame>>> {
        Err(KernelError::Unimplemented)
    }

    /// Checks if the frame exists in this area.
    fn check_frame(&self, _index: usize) -> bool {
        false
    }

    /// Deallocate frame by index.
    fn dealloc_frame(&mut self, _index: usize) -> KernelResult {
        Err(KernelError::Unimplemented)
    }

    /// Splits an area.
    ///
    /// Six cases in total:
    /// 1. `(None, None)`: do nothing
    /// 2. `(start, None)`: split right
    /// 3. `(None, end)`: split left
    /// 4. `(start, end)`: three pieces
    ///
    /// # Argument
    /// - `start`: starting index.
    /// - `end`: ending index.
    ///
    /// # Return
    ///
    /// The first area related to the given range:
    /// - middle part in case 4
    /// - right part in case 2
    /// - left part in case 3
    ///
    /// The second area is the third part in case 4.
    fn split(
        &mut self,
        _start: Option<usize>,
        _end: Option<usize>,
    ) -> KernelResult<(Option<PMA>, Option<PMA>)> {
        Err(KernelError::Unimplemented)
    }

    /// Extends this area.
    ///
    /// Fixed areas cannot be extended with contiguous frames.
    fn extend(&mut self, _new_size: usize) -> KernelResult {
        Err(KernelError::Unimplemented)
    }
}
