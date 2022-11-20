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

    /// Gets the relative path without the first `'/'`.
    pub fn rela(&self) -> &str {
        if self.0.len() == 1 {
            ""
        } else {
            &self.0.as_str()[1..]
        }
    }

    /// Root path
    pub fn is_root(&self) -> bool {
        self.0.len() == 1
    }

    /// Canonicalizes this path.
    ///
    /// 1. Removes `"."` and `".."`;
    /// 2. Replaces contiguous `'/'`s with a single `'/'`.
    pub fn canonicalize(&mut self) {
        self.0 = self
            .split()
            .iter()
            .fold(String::new(), |path, &item| path + "/" + item);
    }

    /// Extends current path with another path. This function is used for seperating
    /// directory path and file name.
    ///
    /// Returns the last item of the input path if `'/'` is not the last character of input path.
    /// Returns empty item if the input path is implied as a directory path (which ends with `'/'`).
    ///
    /// Path may contain `".."`, thus `self` may be shroted after this function called.
    ///
    /// This path will be considered as a directory path.
    pub fn extend_get<'a>(&mut self, path: &'a str) -> &'a str {
        let pos = self.extend(path);
        &path[pos..]
    }

    /// Extends current path with another path. This function is used for seperating
    /// directory path and file name.
    ///
    /// Returns one more than the position of the last `'/'` of the input path if `'/'` is not
    /// the last character of input path.
    ///
    /// Path may contain `".."`, thus `self` may be shroted after this function called.
    ///
    /// This path will be considered as a directory path.
    pub fn extend(&mut self, path: &str) -> usize {
        let mut pos = 0;
        loop {
            if let Some(new_pos) = (&path[pos..]).find('/') {
                if new_pos == 2 && &path[pos..pos + new_pos] == ".." {
                    if self.0.ends_with('/') {
                        self.0.pop();
                    }
                    // Cannot pop the first `'/'`.
                    while self.0.len() > 1 && self.0.pop() != Some('/') {}
                } else if new_pos == 1 && &path[pos..pos + new_pos] == "." {
                    // Do nothing
                } else if new_pos != 0 {
                    if !self.0.ends_with('/') {
                        self.0.push('/');
                    }
                    self.0 += &path[pos..pos + new_pos];
                }
                pos += new_pos + 1;
            } else {
                break pos;
            }
        }
    }

    /// Appends a path item.
    ///
    /// This path will be considered as a directory path.
    pub fn join(&mut self, item: &str) {
        if !self.0.ends_with("/") {
            self.0.push('/');
        }
        self.0 += item;
    }

    /// Gets the last item of this path.
    pub fn last(&mut self) -> Option<String> {
        if self.0.ends_with('/') {
            self.0.pop();
        }
        let pos = self.0.rfind('/').unwrap();
        if pos == 0 {
            None
        } else {
            Some(String::from(&self.0.as_str()[pos + 1..]))
        }
    }

    /// Gets the last item of this path and remove it.
    pub fn pop(&mut self) -> Option<String> {
        if self.0.len() == 1 {
            None
        } else {
            if self.0.ends_with('/') {
                self.0.pop();
            }
            let pos = self.0.rfind('/').unwrap() + 1;
            let item = String::from(&self.0.as_str()[pos..]);
            self.0.drain(pos..);
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
