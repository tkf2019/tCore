#![allow(unused)]

use core::str::FromStr;

use alloc::{string::String, vec::Vec};

/// A wrapper for an absolute path which starts with `'/'` but ends with no `'/'`.
///
/// An item of this path may be a `"."` or `".."` which will not be ignored
/// unless `new_canonical` is called when `Self` is created, or `canonicalize`
/// is called.
///
/// The last item in this path may refer to an existing directory or file,
/// depending on implementations.
///
/// The parent of root direcotry `"/"` is the root itself.
/// E.g. `"/../../.."` is the same as  `"/"`.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Path(String);

impl Path {
    /// Creates a new canonical (No `"."`, `".."` and contiguous `'/'`) path.
    pub fn new(path: &str) -> Self {
        let mut new_path = Self(if path.starts_with("/") {
            String::from(path)
        } else {
            String::from("/") + path
        });
        new_path.canonicalize();
        new_path
    }

    /// Creates a root path.
    pub fn root() -> Self {
        Self::new("/")
    }

    /// Gets the relative path without the first `'/'`.
    pub fn rela(&self) -> &str {
        if self.0.len() == 1 {
            ""
        } else {
            &self.0.as_str()[1..]
        }
    }

    /// Extracts a string slice containing the entire `Path`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Root path
    pub fn is_root(&self) -> bool {
        self.0.len() == 1
    }

    pub fn is_dir(&self) -> bool {
        self.0.ends_with("/")
    }

    /// Canonicalizes this path.
    ///
    /// 1. Removes `"."` and `".."`;
    /// 2. Replaces contiguous `'/'`s with a single `'/'`.
    pub fn canonicalize(&mut self) {
        let is_dir = self.is_dir();
        self.0 = self
            .split()
            .iter()
            .fold(String::new(), |path, &item| path + "/" + item);
        if is_dir {
            self.0.push('/');
        }
    }

    /// Extends current path with another path. This function is used for seperating
    /// directory path and file name.
    ///
    /// Returns one more than the position of the last `'/'` of the input path if `'/'` is not
    /// the last character of input path. Otherwise, returns the length of current directory path.
    ///
    /// Path may contain `".."`, thus `self` may be shroted after this function called.
    ///
    /// This path will be considered as a directory path.
    pub fn extend(&mut self, path: &str) {
        assert!(self.is_dir());
        let mut pos = 0;
        self.0 += path;
        self.canonicalize();
    }

    /// Appends a path item.
    ///
    /// This path will be considered as a directory path.
    pub fn join(&mut self, item: &str) {
        let is_dir = self.is_dir();
        if !is_dir {
            self.0.push('/');
        }
        self.0 += item;
        self.0.push('/');
        self.canonicalize();
    }

    /// Gets the last item of this path.
    /// 
    /// The item will ends with '/' if the original path is a directory.
    pub fn last(&mut self) -> Option<String> {
        if self.is_root() {
            return None;
        }
        let is_dir = self.is_dir();
        if is_dir {
            self.0.pop();
        }
        let pos = self.0.rfind('/').unwrap() + 1;
        if is_dir {
            self.0.push('/');
        }
        Some(String::from(&self.0.as_str()[pos..]))
    }

    /// Gets the last item of this path and remove it.
    /// 
    /// The item will ends with '/' if the original path is a directory.
    pub fn pop(&mut self) -> Option<String> {
        if self.0.len() == 1 {
            None
        } else {
            let is_dir = self.0.ends_with('/');
            if is_dir {
                self.0.pop();
            }
            let pos = self.0.rfind('/').unwrap() + 1;
            let mut item = String::from(&self.0.as_str()[pos..]);
            self.0.drain(pos..);
            if is_dir {
                item.push('/');
            }
            Some(item)
        }
    }

    /// Splits the path into a vector of items.
    ///
    /// 1. Removes `"."` and `".."`;
    /// 2. Replaces contiguous `'/'`s with a single `'/'`.
    pub fn split(&self) -> Vec<&str> {
        self.0.split('/').fold(Vec::with_capacity(8), |mut v, s| {
            match s {
                "" | "." => {}
                ".." => {
                    // Do nothing when trying to find the parent of root "/".
                    v.pop();
                }
                _ => v.push(s),
            }
            v
        })
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        let mut path = Self(value);
        path.canonicalize();
        path
    }
}
