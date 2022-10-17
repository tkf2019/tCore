#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::option::Option::Some;

/// Allocate identifications using different algorithms
pub trait IDAllocator {
    fn alloc(&mut self) -> usize;
    fn dealloc(&mut self, id: usize);
}

pub struct RecycleAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl RecycleAllocator {
    pub fn new() -> Self {
        Self {
            current: 0,
            recycled: Vec::new(),
        }
    }
}

impl IDAllocator for RecycleAllocator {
    fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            self.current += 1;
            self.current - 1
        }
    }

    fn dealloc(&mut self, id: usize) {
        self.recycled.push(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_alloc() {
        let mut r = RecycleAllocator::new();
        assert_eq!(r.alloc(), 0);
        assert_eq!(r.alloc(), 1);
        assert_eq!(r.alloc(), 2);
        r.dealloc(1);
        assert_eq!(r.alloc(), 1);
    }
}
