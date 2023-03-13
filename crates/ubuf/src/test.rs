use crate::*;
use core::slice;

extern crate std;

#[repr(C)]
#[derive(Debug)]
pub struct A {
    pub a: u8,
    pub b: u32,
    pub c: u16,
}

#[test]
#[allow(unused_unsafe)]
fn test_read() {
    unsafe {
        let mut v = Vec::new();
        v.push(slice::from_raw_parts_mut("aasdss".as_ptr() as *mut u8, 6));
        v.push(slice::from_raw_parts_mut(
            "asdaf1234a".as_ptr() as *mut u8,
            10,
        ));
        let ubuf = UserBuffer::new(v);
        let mut a = A { a: 0, b: 0, c: 0 };
        read_user_buf!(ubuf, A, a);
        std::println!("{:x?}", a);

        let mut v = Vec::new();
        v.push(slice::from_raw_parts_mut("aasdss".as_ptr() as *mut u8, 6));
        v.push(slice::from_raw_parts_mut(
            "asdaf1234a".as_ptr() as *mut u8,
            10,
        ));
        let ubuf = UserBuffer::new(v);
        let mut a = A { a: 0, b: 0, c: 0 };
        read_user_buf!(ubuf, 8, a);
        std::println!("{:x?}", a);
    }
}
