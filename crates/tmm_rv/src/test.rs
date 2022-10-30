extern crate std;

use std::println;

use alloc::collections::BTreeMap;

use crate::{frame_alloc::GLOBAL_FRAME_ALLOCATOR, *};

#[test]
fn test_frame_alloc() {
    frame_init(111, 300);
    println!("{}", frame_alloc(1).unwrap());
    println!("{}", frame_alloc(5).unwrap());
    frame_dealloc(111, 7);
    println!("{}", frame_alloc(2).unwrap());
}
