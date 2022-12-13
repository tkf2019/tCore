use alloc::{sync::Arc, vec::Vec};
use core::fmt;
use spin::Mutex;
use tmm_rv::{AllocatedFrame, Frame, PAGE_SIZE};

use crate::{
    config::USER_MAX_PAGES,
    error::{KernelError, KernelResult},
};

use super::{BackendFile, PMArea, PMA};

/// Physical memory mapped but not allocated with real memory space.
pub struct LazyPMA {
    /// Allocated frames from global allocator. This area has the ownership
    /// of these frames. The physical frames will be deallocated if this lazy
    /// area is unmapped from virtual areas and dropped.
    frames: Vec<Option<AllocatedFrame>>,

    /// The number of allocated frames.
    alloc_count: usize,

    /// Data can be loaded from or stored to this file.
    ///
    /// Current offset of target file must be aligned to page size.
    file: Option<BackendFile>,
}

impl LazyPMA {
    /// Creates a new lazy area.
    pub fn new(count: usize, file: Option<BackendFile>) -> KernelResult<Self> {
        if count == 0 || count > USER_MAX_PAGES {
            return Err(KernelError::InvalidArgs);
        }
        let mut frames = Vec::with_capacity(count);
        frames.fill_with(|| None);
        Ok(Self {
            frames,
            alloc_count: 0,
            file,
        })
    }

    /// Creates a new lazy area with allocated frames, usually generated from
    /// another area.
    pub fn new_with_frames(
        frames: Vec<Option<AllocatedFrame>>,
        file: Option<BackendFile>,
    ) -> KernelResult<Self> {
        let mut alloc_count = 0;
        frames
            .iter()
            .for_each(|frame| alloc_count += frame.is_some() as usize);
        Ok(Self {
            frames,
            alloc_count,
            file,
        })
    }
}

impl PMArea for LazyPMA {
    fn is_mapped(&self) -> bool {
        true
    }

    fn get_frame(&mut self, index: usize, alloc: bool) -> KernelResult<Frame> {
        if let Some(frame) = &self.frames[index] {
            Ok((*frame).clone())
        } else if alloc {
            let frame = AllocatedFrame::new(true).map_err(|_| KernelError::FrameAllocFailed)?;
            if let Some(file) = &self.file {
                if file.read(index * PAGE_SIZE, frame.as_slice_mut()).is_none() {
                    return Err(KernelError::PMAFailedIO);
                }
            }
            let frame_inner = frame.clone();
            // ownership moved
            self.frames[index] = Some(frame);
            self.alloc_count += 1;
            Ok(frame_inner)
        } else {
            Err(KernelError::PMAFrameNotFound)
        }
    }

    fn get_frames(&mut self, alloc: bool) -> KernelResult<Vec<Option<Frame>>> {
        let mut v = Vec::new();
        for frame in &mut self.frames {
            if let Some(frame) = frame {
                v.push(Some((*frame).clone()));
            } else {
                if alloc {
                    let new_frame = frame.insert(
                        AllocatedFrame::new(true).map_err(|_| KernelError::FrameAllocFailed)?,
                    );
                    v.push(Some(new_frame.clone()));
                    self.alloc_count += 1;
                } else {
                    v.push(None);
                }
            }
        }
        Ok(v)
    }

    fn check_frame(&self, index: usize) -> bool {
        self.frames[index].is_some()
    }

    fn dealloc_frame(&mut self, index: usize) -> KernelResult {
        let frame = self.frames[index]
            .take()
            .ok_or(KernelError::PMAFrameNotFound)?;
        // Write back dirty frame.
        if let Some(file) = &self.file {
            file.write(index * PAGE_SIZE, frame.as_slice());
        }
        self.alloc_count -= 1;
        Ok(())
    }

    fn split(
        &mut self,
        start: Option<usize>,
        end: Option<usize>,
    ) -> KernelResult<(Option<PMA>, Option<PMA>)> {
        if start.is_none() && end.is_none() {
            Ok((None, None))
        } else if end.is_none() {
            let start = start.unwrap();

            if start >= self.frames.len() {
                return Err(KernelError::PMAOutOfRange);
            }

            Ok((
                Some(Arc::new(Mutex::new(Self::new_with_frames(
                    self.frames.drain(start..).collect(),
                    self.file.as_ref().map(|file| file.split(start * PAGE_SIZE)),
                )?))),
                None,
            ))
        } else if start.is_none() {
            let end = end.unwrap();

            if end > self.frames.len() {
                return Err(KernelError::PMAOutOfRange);
            }

            // Update file offset.
            if let Some(file) = &mut self.file {
                file.seek(end * PAGE_SIZE);
            };

            Ok((
                Some(Arc::new(Mutex::new(Self::new_with_frames(
                    self.frames.drain(..end).collect(),
                    self.file.as_ref().map(|file| file.split(0)),
                )?))),
                None,
            ))
        } else {
            let start = start.unwrap();
            let end = end.unwrap();

            if start == end {
                return Ok((None, None));
            }

            if start > end {
                return Err(KernelError::InvalidArgs);
            }

            if start >= self.frames.len() || end >= self.frames.len() {
                return Err(KernelError::PMAOutOfRange);
            }

            let right_pma = Arc::new(Mutex::new(Self::new_with_frames(
                self.frames.drain(end..).collect(),
                self.file.as_ref().map(|file| file.split(end * PAGE_SIZE)),
            )?));
            let mid_pma = Arc::new(Mutex::new(Self::new_with_frames(
                self.frames.drain(start..).collect(),
                self.file.as_ref().map(|file| file.split(start * PAGE_SIZE)),
            )?));

            Ok((Some(mid_pma), Some(right_pma)))
        }
    }

    fn extend(&mut self, new_size: usize) -> KernelResult {
        self.frames.resize_with(new_size, || None);
        Ok(())
    }
}

impl fmt::Debug for LazyPMA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mapped memory area: [{:?}]", self.frames)
    }
}
