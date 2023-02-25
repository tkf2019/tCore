//! A naive sleeping mutex.
//!
//! A preemptive kernel differs from a nonpreemptive kernel on the way a process running in Kernel Mode
//! reacts to asynchronous events that could induce a process switch—for instance, an interrupt handler
//! that awakes a higher priority process. We will call this kind of process switch a forced process switch.
//!
//! Thus we use sleep lock to avoid infinite latency.

use core::{
    cell::UnsafeCell,
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
};

use id_alloc::{IDAllocator, RecycleAllocator};
use spin::Lazy;

use crate::{
    arch::{cpu_id, intr_get},
    spinlock::{SpinLock, SpinLockGuard},
    CPUs,
};

/// Threads should implement this trait to support sleep lock.
pub trait Sched {
    /// Thread state is changed to `SLEEPING`.
    fn sleep(thread: &mut Self);

    /// Wakeup all threads sleeping on this lock.
    fn wakeup(id: usize);

    /// Threads acquiring this [`SleepLock`] is grouped by the id.
    fn set_id(thread: &mut Self, id: Option<usize>);

    /// Switch to scheduler.
    unsafe fn sched();
}

static SleepLockIDAllocator: Lazy<SpinLock<RecycleAllocator>> =
    Lazy::new(|| SpinLock::new(RecycleAllocator::new(0)));

/// A sleep lock providing mutually exclusive access to data and yielding the CPU when locked.
pub struct SleepLock<T: ?Sized, S: Sched> {
    phantom: PhantomData<S>,

    /// [`SpinLock`] protecting this [`SleepLock`].
    inner: SpinLock<SleepLockInner<T, S>>,
}

/// Inner info protected by lock.
pub struct SleepLockInner<T: ?Sized, S: Sched> {
    phantom: PhantomData<S>,

    /// A unique identifier of this [`SleepLock`].
    id: usize,

    /// If this mutex is locked and holds the inner data.
    locked: bool,

    /// Data of this mutex.
    data: UnsafeCell<T>,
}

/// A guard that provides mutable data access.
///
/// When the guard falls out of scope it will release the lock.
pub struct SleepLockGuard<'a, T: ?Sized + 'a, S: Sched> {
    phantom: PhantomData<S>,
    lock: &'a SpinLock<SleepLockInner<T, S>>,
    data: &'a mut T,
}

// unsafe thread-safe impls
unsafe impl<T: ?Sized + Send, S: Sched> Sync for SleepLock<T, S> {}
unsafe impl<T: ?Sized + Send, S: Sched> Send for SleepLock<T, S> {}

impl<T, S: Sched> SleepLock<T, S> {
    /// Creates a new [`SleepLock`] wrapping the supplied data.
    #[inline(always)]
    pub fn new(data: T) -> Self {
        SleepLock {
            phantom: PhantomData,
            inner: SpinLock::new(SleepLockInner::new(data)),
        }
    }

    /// Consumes this [`SleepLock`] and unwraps the underlying data.
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let SleepLock { inner, .. } = self;
        inner.into_inner().into_inner()
    }

    /// Returns a mutable pointer to the underlying data.
    ///
    /// This is mostly meant to be used for applications which require manual unlocking, but where
    /// storing both the lock and the pointer to the inner data gets inefficient.
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        unsafe { (*self.inner.as_mut_ptr()).as_mut_ptr() }
    }
}

impl<T, S: Sched> SleepLockInner<T, S> {
    /// Creates a new [`SleepLockInner`].
    #[inline(always)]
    pub fn new(data: T) -> Self {
        SleepLockInner {
            phantom: PhantomData,
            id: SleepLockIDAllocator.lock().alloc(),
            locked: false,
            data: UnsafeCell::new(data),
        }
    }

    /// Consumes this [`SleepLockInner`] and unwraps the underlying data.
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let SleepLockInner { data, .. } = self;
        data.into_inner()
    }

    /// Returns a mutable pointer to the underlying data.
    ///
    /// This is mostly meant to be used for applications which require manual unlocking, but where
    /// storing both the lock and the pointer to the inner data gets inefficient.
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.data.get()
    }
}

impl<T: ?Sized, S: Sched> SleepLock<T, S> {
    /// Locks the [`SleepLock`], returning a guard that permits access to the inner data.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    #[inline(always)]
    pub fn lock(&self, thread: &SpinLock<S>) -> SleepLockGuard<T, S> {
        let mut inner = self.inner.lock();
        let lock_id = inner.id;

        // Automatically release the lock and sleep on chan.
        while inner.locked {
            // Must acquire thread lock in order to change thread state and then call sched().
            let mut guard = thread.lock();
            drop(inner);

            // Go to sleep
            S::sleep(&mut guard);
            S::set_id(&mut guard, Some(lock_id));

            unsafe {
                // Interrupt cannot be nesting or set before scheduler.
                assert!(CPUs[cpu_id()].noff == 1 && !intr_get());
                // Saves and restores CPU local variable, intena.
                let intena = CPUs[cpu_id()].intena;
                S::sched();
                CPUs[cpu_id()].intena = intena;
            }

            // Tidy up
            S::set_id(&mut guard, None);

            // Reacquire original lock
            drop(guard);
            inner = self.inner.lock();
        }
        inner.locked = true;

        SleepLockGuard {
            phantom: PhantomData,
            lock: &self.inner,
            data: unsafe { &mut *inner.data.get() },
        }
    }

    /// Returns `true` if the lock is currently held.
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.inner.lock().locked
    }

    /// Force unlock this [`SleepLock`].
    #[inline(always)]
    pub unsafe fn force_unlock(&self) {
        let mut inner = self.inner.lock();
        inner.locked = false;
        S::wakeup(inner.id);
    }

    /// Tries to lock this [`SleepLock`], returning a guard if successful.
    #[inline(always)]
    pub fn try_lock(&self) -> Option<SleepLockGuard<T, S>> {
        let mut inner = self.inner.lock();
        if !inner.locked {
            inner.locked = true;
            Some(SleepLockGuard {
                phantom: PhantomData,
                lock: &self.inner,
                data: unsafe { &mut *inner.data.get() },
            })
        } else {
            None
        }
    }
}

impl<T: ?Sized + fmt::Debug, S: Sched> fmt::Debug for SleepLock<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "SleepLock {{ data: ")
                .and_then(|()| (&*guard).fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "SleepLock {{ <locked> }}"),
        }
    }
}

impl<T: ?Sized + Default, S: Sched> Default for SleepLock<T, S> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T, S: Sched> From<T> for SleepLock<T, S> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<T: ?Sized, S: Sched> SleepLockInner<T, S> {
    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the [`SleepLockInner`] mutably, and a mutable reference is guaranteed to be exclusive in Rust,
    /// no actual locking needs to take place -- the mutable borrow statically guarantees no locks exist. As such,
    /// this is a 'zero-cost' operation.
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        // We know statically that there are no other references to `self`, so
        // there's no need to lock the inner mutex.
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized + fmt::Debug, S: Sched> fmt::Debug for SleepLockGuard<'a, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized, S: Sched> Deref for SleepLockGuard<'a, T, S> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized, S: Sched> DerefMut for SleepLockGuard<'a, T, S> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized, S: Sched> Drop for SleepLockGuard<'a, T, S> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self) {
        let mut inner = self.lock.lock();
        inner.locked = false;
        // Wake up all processes sleeping on chan.
        // Must be called without any thread lock.
        S::wakeup(inner.id);
    }
}
