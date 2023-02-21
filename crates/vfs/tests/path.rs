extern crate std;

use std::println;

use vfs::{OpenFlags, Path};

#[test]
fn test_open_flags() {
    println!("{:#?}", OpenFlags::O_APPEND);
}

#[test]
fn test_path() {
    let mut path = Path::new("a/b/c/.././..///d/");
    assert_eq!(path, Path::new("/a/d/"));
    path.extend("a/b/c/.././..///test.txt");
    let last = path.last().unwrap();
    assert_eq!(last, "test.txt");
    assert!(!path.is_dir());
    path.pop().unwrap();
    assert!(path.is_dir());
    assert_eq!(path, Path::new("/a/d/a/"));
    path.extend("a/b/c/.././..///d/");
    assert_eq!(path, Path::new("/a/d/a/a/d/////"));
    assert_ne!(path, Path::new("/a/d/a/a/d"))
}
