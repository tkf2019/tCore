//! A naive spinning mutex.
//!
//! Waiting threads hammer an atomic variable until it becomes available. Best-case latency is low, but worst-case
//! latency is theoretically infinite.

use core::{
    cell::UnsafeCell,
    fmt,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{pop_off, push_off};

/// A [spin lock](https://en.m.wikipedia.org/wiki/Spinlock) providing mutually exclusive access to data.
pub struct SpinLock<T: ?Sized> {
    pub(crate) lock: AtomicBool,
    data: UnsafeCell<T>,
}

/// A guard that provides mutable data access.
///
/// When the guard falls out of scope it will release the lock.
pub struct SpinLockGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<T: ?Sized + Send> Sync for SpinLock<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Creates a new [`SpinLock`] wrapping the supplied data.
    #[inline(always)]
    pub const fn new(data: T) -> Self {
        SpinLock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Consumes this [`SpinLock`] and unwraps the underlying data.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let SpinLock { data, .. } = self;
        data.into_inner()
    }
}

impl<T: ?Sized> SpinLock<T> {
    /// Locks the [`SpinLock`] and returns a guard that permits access to the inner data.
    #[inline(always)]
    pub fn lock(&self) -> SpinLockGuard<T> {
        // Disable interrrupts to avoid deadlock.
        push_off();
        // Can fail to lock even if the spinlock is not locked. May be more efficient than `try_lock`
        // when called in a loop.
        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Wait until the lock looks unlocked before retrying
            while self.is_locked() {
                core::hint::spin_loop();
            }
        }

        SpinLockGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Returns `true` if the lock is currently held.
    ///
    /// # Safety
    ///
    /// This function provides no synchronization guarantees and so its result should be considered 'out of date'
    /// the instant it is called. Do not use it for synchronization purposes. However, it may be useful as a heuristic.
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    /// Force unlock this [`SpinLock`].
    ///
    /// # Safety
    ///
    /// This is *extremely* unsafe if the lock is not held by the current
    /// thread. However, this can be useful in some instances for exposing the
    /// lock to FFI that doesn't know how to deal with RAII.
    #[inline(always)]
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
        // Back to previous interrupt enabling bit.
        pop_off();
    }

    /// Try to lock this [`Mutex`], returning a lock guard if successful.
    #[inline(always)]
    pub fn try_lock(&self) -> Option<SpinLockGuard<T>> {
        // Disable interrrupts to avoid deadlock.
        push_off();
        // The reason for using a strong compare_exchange is explained here:
        // https://github.com/Amanieu/parking_lot/pull/207#issuecomment-575869107
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinLockGuard {
                lock: &self.lock,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            // Failed to acquire the lock.
            // Back to previous interrupt enabling bit.
            pop_off();
            None
        }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the [`SpinLock`] mutably, and a mutable reference is guaranteed to be exclusive in
    /// Rust, no actual locking needs to take place -- the mutable borrow statically guarantees no locks exist.
    /// As such, this is a 'zero-cost' operation.
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        // We know statically that there are no other references to `self`, so
        // there's no need to lock the inner data.
        unsafe { &mut *self.data.get() }
    }

    /// Returns a mutable pointer to the underlying data.
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.data.get()
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "SpinLock {{ data: ")
                .and_then(|()| (&*guard).fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "SpinLock {{ <locked> }}"),
        }
    }
}

impl<T: ?Sized + Default> Default for SpinLock<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> From<T> for SpinLock<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<'a, T: ?Sized> SpinLockGuard<'a, T> {
    /// Leak the lock guard, yielding a mutable reference to the underlying data.
    ///
    /// Note that this function will permanently lock the original [`SpinLock`].
    #[inline(always)]
    pub fn leak(this: Self) -> &'a mut T {
        let data = this.data as *mut _; // Keep it in pointer form temporarily to avoid double-aliasing
        core::mem::forget(this);
        unsafe { &mut *data }
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for SpinLockGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for SpinLockGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinLockGuard<'a, T> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
        // Back to previous interrupt enabling bit.
        pop_off();
    }
}
