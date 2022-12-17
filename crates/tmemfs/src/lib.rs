#![no_std]
#![allow(unused)]

extern crate alloc;

use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;
use tmm_rv::{AllocatedFrame, Frame, PAGE_SIZE};
use tvfs::{File, SeekWhence};

mod null;
mod zero;

struct MemFileInner {
    /// Allocated frames to store file data temporarily.
    frames: Vec<AllocatedFrame>,

    /// Current position of the cursor.
    pos: usize,
}

impl MemFileInner {
    pub fn new(limit: usize) -> Self {
        let mut frames = Vec::with_capacity(limit / PAGE_SIZE);
        frames.fill_with(|| AllocatedFrame::new(false).unwrap());
        Self { frames, pos: 0 }
    }
}

/// File object that stored in memory usually created during the
/// initialization of an operating system.
///
/// The file size must be multiple of PAGE_SIZE.
pub struct MemFile {
    inner: Mutex<MemFileInner>,

    /// Max size of this file.
    max_size: usize,
}

impl MemFile {
    /// Creates an empty memory file.
    pub fn new(limit: usize) -> Self {
        Self {
            inner: Mutex::new(MemFileInner::new(limit)),
            max_size: limit,
        }
    }

    /// Gets the limit size of this file.
    pub fn get_limit(&self) -> usize {
        self.max_size
    }
}

impl File for MemFile {
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        let mut inner = self.inner.lock();
        let read_len = buf.len().min(self.max_size - inner.pos);
        let read_end = inner.pos + read_len;
        let start_idx = inner.pos / PAGE_SIZE;
        let end_idx = (read_end - 1) / PAGE_SIZE;
        for i in start_idx..=end_idx {
            let frame = inner.frames[i].as_slice();
            let off = inner.pos & (PAGE_SIZE - 1);
            let len = (PAGE_SIZE - off).min(read_end - inner.pos);
            buf.copy_from_slice(&frame[off..off + len]);
            inner.pos += len;
        }
        Some(read_len)
    }

    fn write(&self, buf: &[u8]) -> Option<usize> {
        let mut inner = self.inner.lock();
        let write_len = buf.len().min(self.max_size - inner.pos);
        let write_end = inner.pos + write_len;
        let start_idx = inner.pos / PAGE_SIZE;
        let end_idx = (write_end - 1) / PAGE_SIZE;
        for i in start_idx..=end_idx {
            let frame = inner.frames[i].as_slice_mut();
            let off = inner.pos & (PAGE_SIZE - 1);
            let len = (PAGE_SIZE - off).min(write_end - inner.pos);
            (&mut frame[off..off + len]).copy_from_slice(buf);
            inner.pos += len;
        }
        Some(write_len)
    }

    fn seek(&self, offset: usize, whence: SeekWhence) -> Option<usize> {
        let mut inner = self.inner.lock();
        match whence {
            SeekWhence::Current => {
                let new_pos = inner.pos as isize + offset as isize;
                if new_pos < 0 || new_pos > self.max_size as isize {
                    return None;
                }
                inner.pos = new_pos as usize;
            }
            SeekWhence::Set => {
                inner.pos = offset;
            }
            SeekWhence::End => {
                let new_pos = self.max_size as isize + offset as isize;
                if new_pos < 0 || new_pos > self.max_size as isize {
                    return None;
                }
                inner.pos = new_pos as usize;
            }
        };
        Some(inner.pos)
    }

    fn get_size(&self) -> Option<usize> {
        Some(self.inner.lock().frames.len() * PAGE_SIZE)
    }

    fn get_off(&self) -> usize {
        self.inner.lock().pos
    }
}
