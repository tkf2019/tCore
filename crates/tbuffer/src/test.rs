use std::println;

use spin::Mutex;
use tvfs::File;

use crate::RingBuffer;

extern crate std;

const FILE_SIZE: usize = 512;

struct TestFile {
    data: Mutex<[u8; FILE_SIZE]>,
    pos: Mutex<usize>,
}

impl TestFile {
    pub fn new() -> Self {
        Self {
            data: Mutex::new([0; FILE_SIZE]),
            pos: Mutex::new(0),
        }
    }
}

impl File for TestFile {
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        let mut pos = self.pos.lock();
        let data = self.data.lock();
        let read_len = buf.len().min(FILE_SIZE - *pos);
        buf.copy_from_slice(&data[*pos..*pos + read_len]);
        *pos += read_len;
        Some(read_len)
    }

    fn write(&self, buf: &[u8]) -> Option<usize> {
        let mut pos = self.pos.lock();
        let mut data = self.data.lock();
        let write_len = buf.len().min(FILE_SIZE - *pos);
        (&mut data[*pos..*pos + write_len]).copy_from_slice(buf);
        *pos += write_len;
        Some(write_len)
    }

    fn seek(&self, offset: usize, whence: tvfs::SeekWhence) -> Option<usize> {
        let mut pos = self.pos.lock();
        match whence {
            tvfs::SeekWhence::Set => {
                *pos = offset;
            }
            tvfs::SeekWhence::Current => {
                *pos += offset;
            }
            tvfs::SeekWhence::End => {
                *pos = FILE_SIZE + offset;
            }
        }
        Some(*pos)
    }
}

#[test]
fn test_ring_buffer() {
    let mut buffer = RingBuffer::new(FILE_SIZE, TestFile::new());
    buffer.write("abcdefghijk".as_bytes());
    let mut buf = [0u8; 512];
    buffer.read(&mut buf);
    assert!(buffer.is_empty());
    assert_eq!(buf[0], 'a' as u8);
    println!("OK {}", alloc::str::from_utf8(&buf[..3]).unwrap());
}
