#![no_std]
#![allow(unused)]
#![feature(linked_list_remove)]

extern crate alloc;

mod block;

use core::any::Any;

pub use block::*;

pub trait CacheUnit: Send + Sync + Any {
    /// Synchronize data in this block to the next level of memory system.
    fn sync(&mut self);

    /// Get the address with the offset in this cache unit.
    ///
    /// The address might be `virtual` or `physical`, which depends on the memory layout
    /// of the cache manager.
    fn addr(&self, offset: usize) -> usize;

    /// Make this cache unit dirty, which means this cache need to be synchronized to
    /// the next level of memory system.
    fn set_dirty(&mut self);

    /// The size of this unit in bytes
    fn size(&self) -> usize;

    /// The cache unit must be able to be located with an index in the next level of memory system.
    ///
    /// # Examples
    /// - A block can be located with the block id, or sector id.
    /// - A physical frame can be located with the physical frame number.
    fn id(&self) -> usize;

    /// Get a immutable reference of the given object from the offset in this cache unit.
    ///
    /// # Panic
    /// - The size of inferred object exceeds the size of this cache unit.
    fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let tyep_size = core::mem::size_of::<T>();
        assert!(offset + tyep_size <= self.size());
        let addr = self.addr(offset);
        unsafe { &*(addr as *const T) }
    }

    /// Get a mutable reference of the given object from the offset in this cache unit.
    ///
    /// # Panic
    /// - The size of inferred object exceeds the size of this cache unit.
    fn get_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let tyep_size = core::mem::size_of::<T>();
        assert!(offset + tyep_size <= self.size());
        let addr = self.addr(offset);
        self.set_dirty();
        unsafe { &mut *(addr as *mut T) }
    }

    /// Read-only and read once.
    ///
    /// # Argument
    /// - `offset`: offset in this cache unit.
    /// - `f`: a function called once and does immutable operations on
    /// the object in this cache unit.
    ///
    /// # Panic
    /// - The size of inferred object exceeds the size of this cache unit.
    fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    /// Write once.
    ///
    /// # Argument
    /// - `offset`: offset in this cache unit.
    /// - `f`: a function called once and does immutable operations on
    /// the object in this cache unit.
    ///
    /// # Panic
    /// - The size of inferred object exceeds the size of this cache unit.
    fn write<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }
}
