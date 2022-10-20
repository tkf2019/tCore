extern crate std;

use std::println;

use crate::frame::FRAME_ALLOCATOR;

use super::*;

#[test]
fn test_page_table() {
    FRAME_ALLOCATOR.lock().add_frame(0, 20);
    let page_table = PageTable::new().unwrap();
    println!("{:#?}", page_table);
}
