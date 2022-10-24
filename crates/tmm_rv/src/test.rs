extern crate std;

use std::println;

use alloc::collections::BTreeMap;

use crate::*;

#[test]
fn test_frame_alloc() {
    frame_alloc::init(0, 100);
    println!("{}", frame_alloc(1).unwrap());
    println!("{}", frame_alloc(5).unwrap());
    frame_dealloc(0, 2);
    println!("{}", frame_alloc(2).unwrap());
}

#[test]
fn test_btree() {
}
