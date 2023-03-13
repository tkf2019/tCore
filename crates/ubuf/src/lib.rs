#![no_std]

#[cfg(test)]
mod test;

extern crate alloc;

use alloc::vec::Vec;
use core::iter::{IntoIterator, Iterator};

pub struct UserBuffer {
    pub inner: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    /// Creates a new buffer with inner data.
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { inner: buffers }
    }
}

pub struct UserBufferIterator {
    inner: Vec<&'static mut [u8]>,
    curr_buf: usize,
    curr_idx: usize,
}

impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            inner: self.inner,
            curr_buf: 0,
            curr_idx: 0,
        }
    }
}

impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_buf >= self.inner.len() {
            None
        } else {
            let r = &mut self.inner[self.curr_buf][self.curr_idx] as *mut _;
            if self.curr_idx + 1 == self.inner[self.curr_buf].len() {
                self.curr_idx = 0;
                self.curr_buf += 1;
            } else {
                self.curr_idx += 1;
            }
            Some(r)
        }
    }
}

#[macro_export]
macro_rules! user_buf_next {
    ($iter:expr, $ty:ty) => {
        unsafe { &*($iter.next().unwrap() as *const $ty) }
    };
}

#[macro_export]
macro_rules! user_buf_next_mut {
    ($iter:expr, $ty:ty) => {
        unsafe { &mut *($iter.next().unwrap() as *mut $ty) }
    };
}

#[macro_export]
macro_rules! write_user_buf {
    ($ubuf:expr, $ty:ty, $buf:expr) => {
        unsafe {
            let iter = $ubuf.into_iter();
            let buf = core::slice::from_raw_parts(
                &$buf as *const _ as *const u8,
                core::mem::size_of::<$ty>(),
            );
            iter.zip(buf.into_iter()).for_each(|(a, b)| {
                *a = *b;
            });
        }
    };

    ($ubuf:expr, $size:expr, $buf:expr) => {
        unsafe {
            let iter = $ubuf.into_iter();
            let buf = core::slice::from_raw_parts(&$buf as *const _ as *const u8, $size);
            iter.zip(buf.into_iter()).for_each(|(a, b)| {
                *a = *b;
            });
        }
    };
}

#[macro_export]
macro_rules! read_user_buf {
    ($ubuf:expr, $ty:ty, $buf:expr) => {
        unsafe {
            let iter = $ubuf.into_iter();
            let buf = core::slice::from_raw_parts_mut(
                &mut $buf as *const _ as *mut u8,
                core::mem::size_of::<$ty>(),
            );
            buf.into_iter().zip(iter).for_each(|(a, b)| {
                *a = *b;
            });
        }
    };

    ($ubuf:expr, $size:expr, $buf:expr) => {
        unsafe {
            let iter = $ubuf.into_iter();
            let buf = core::slice::from_raw_parts_mut(&mut $buf as *const _ as *mut u8, $size);
            buf.into_iter().zip(iter).for_each(|(a, b)| {
                *a = *b;
            });
        }
    };
}
