extern crate std;

use std::println;

use super::*;

#[test]
fn test_open_flags() {
    println!("{:#?}", OpenFlags::O_APPEND);
}
