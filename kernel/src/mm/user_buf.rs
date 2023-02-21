use alloc::vec::Vec;

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
