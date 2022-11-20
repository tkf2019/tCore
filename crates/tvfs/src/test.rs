extern crate std;

use std::println;

use crate::{path::Path, OpenFlags};

#[test]
fn test_open_flags() {
    println!("{:#?}", OpenFlags::O_APPEND);
}

#[test]
fn test_path() {
    let mut path = Path::new("a/b/c/.././..///d/");
    assert_eq!(path, Path::new("/a/d"));
    path.canonicalize();
    assert_eq!(path, Path::new("/a/d"));
    let last = path.extend_get("a/b/c/.././..///test.txt");
    assert_eq!(last, "test.txt");
    let last = path.last().unwrap();
    assert_eq!(last, "a");
    path.pop().unwrap();
    assert_eq!(path, Path::new("/a/d"));
    let last = path.extend_get("a/b/c/.././..///d/");
    assert!(last.is_empty());
}
